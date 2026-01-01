# 应用内购买服务配置指南

## 概述

本指南说明如何为游戏引擎配置应用内购买服务：
- **Android**: Google Play Billing Library
- **iOS**: StoreKit 2.0

## Android Google Play Billing配置

### 1. 创建Google Play Console项目

1. 访问 [Google Play Console](https://play.google.com/console)
2. 创建新应用或选择现有应用
3. 配置应用详情：
   - 应用名称
   - 应用描述
   - 图标和截图

### 2. 配置应用内商品

1. 在Google Play Console中：
   - 导航到"变现" → "产品" → "应用内商品"
2. 创建商品：
   - **消耗型**：游戏货币、道具等（可重复购买）
   - **非消耗型**：去广告、完整版等（一次性购买）
   - **订阅型**：月卡、年卡等（周期性付费）

3. 记录商品ID（如：`coin_pack_100`, `no_ads`, `premium_monthly`）

### 3. 配置build.gradle

**应用级 build.gradle**:
```gradle
dependencies {
    implementation 'com.android.billingclient:billing:6.0.1'
}
```

### 4. 创建Billing FFI桥接类

**BillingFFIWrapper.kt**:
```kotlin
package com.gameengine.mobile

import com.android.billingclient.api.*

class BillingFFIWrapper(private val context: android.content.Context) : PurchasesUpdatedListener {

    private var billingClient: BillingClient = BillingClient.newBuilder(context)
        .setListener(this)
        .enablePendingPurchases()
        .build()

    private var productsCallback: ((List<ProductDetails>) -> Unit)? = null
    private var purchaseCallback: ((String) -> Unit)? = null

    override fun onPurchasesUpdated(billingResult: BillingResult, purchases: MutableList<Purchase>?) {
        if (billingResult.responseCode == BillingClient.BillingResponseCode.OK && purchases != null) {
            for (purchase in purchases) {
                handlePurchase(purchase)
            }
        }
    }

    private fun handlePurchase(purchase: Purchase) {
        val purchaseToken = purchase.purchaseTokens.firstOrNull() ?: return

        if (purchase.purchaseState == Purchase.PurchaseState.PURCHASED) {
            if (!purchase.isAcknowledged) {
                val acknowledgePurchaseParams = AcknowledgePurchaseParams.newBuilder()
                    .setPurchaseToken(purchaseToken)
                    .build()

                billingClient.acknowledgePurchase(acknowledgePurchaseParams) { billingResult ->
                    if (billingResult.responseCode == BillingClient.BillingResponseCode.OK) {
                        purchaseCallback?.invoke(purchaseToken)
                    }
                }
            }
        }
    }

    external fun initialize(): Int {
        return try {
            billingClient.startConnection(object : BillingClientStateListener {
                override fun onBillingSetupFinished(billingResult: BillingResult) {
                    if (billingResult.responseCode == BillingClient.BillingResponseCode.OK) {
                        Log.d("Billing", "Billing client initialized")
                    }
                }

                override fun onBillingServiceDisconnected() {
                    Log.e("Billing", "Billing service disconnected")
                }
            })
            1 // Success
        } catch (e: Exception) {
            0 // Failure
        }
    }

    external fun queryProducts(productIds: Array<String>): String {
        val productList = productIds.map {
            QueryProductDetailsParams.Product.newBuilder()
                .setProductId(it)
                .setProductType(BillingClient.ProductType.INAPP)
                .build()
        }

        val params = QueryProductDetailsParams.newBuilder()
            .setProductList(productList)
            .build()

        billingClient.queryProductDetailsAsync(params) { billingResult, productDetailsList ->
            productsCallback?.invoke(productDetailsList)
        }

        return "" // Actual result via callback
    }

    external fun purchase(productId: String): String {
        val params = BillingFlowParams.ProductDetailsParams.newBuilder()
            .setProductId(productId)
            .build()

        val flowParams = BillingFlowParams.newBuilder()
            .setProductDetailsParamsList(listOf(params))
            .build()

        billingClient.launchBillingFlow(context as Activity, flowParams)
        return "" // Result via callback
    }

    external fun consume(purchaseToken: String): Int {
        val params = ConsumeParams.newBuilder()
            .setPurchaseToken(purchaseToken)
            .build()

        billingClient.consumeAsync(params) { billingResult, _ ->
            Log.d("Billing", "Purchase consumed: ${billingResult.responseCode}")
        }

        return 1 // Success
    }

    external fun restore(): String {
        billingClient.queryPurchasesAsync(
            QueryPurchasesParams.newBuilder()
                .setProductType(BillingClient.ProductType.INAPP)
                .build()
        ) { billingResult, list ->
            // Handle restored purchases
        }
        return "[]"
    }

    companion object {
        init {
            System.loadLibrary("game_engine")
        }
    }
}
```

**JNI桥接实现**:
```kotlin
// BillingJNI.kt
package com.gameengine.mobile

class BillingJNI {

    external fun initialize_ffi(): Int {
        val wrapper = BillingFFIWrapper(context)
        return wrapper.initialize()
    }

    external fun query_products_ffi(productIdsJson: String): String {
        val wrapper = BillingFFIWrapper(context)
        val productIds = Gson().fromJson(productIdsJson, Array<String>::class.java)
        return wrapper.queryProducts(productIds)
    }

    external fun purchase_ffi(productId: String): String {
        val wrapper = BillingFFIWrapper(context)
        return wrapper.purchase(productId)
    }

    external fun consume_ffi(purchaseToken: String): Int {
        val wrapper = BillingFFIWrapper(context)
        return wrapper.consume(purchaseToken)
    }

    external fun restore_ffi(): String {
        val wrapper = BillingFFIWrapper(context)
        return wrapper.restore()
    }
}
```

### 5. AndroidManifest.xml配置

```xml
<manifest>
    <application>
        <!-- Billing权限 -->
        <uses-permission android:name="com.android.vending.BILLING" />
    </application>
</manifest>
```

## iOS StoreKit配置

### 1. 创建App ID和商品

1. 访问 [Apple Developer](https://developer.apple.com/)
2. Certificates, Identifiers & Profiles → Identifiers
3. 创建App ID（启用In-App Purchase能力）
4. 在App Store Connect中配置商品：
   - **消耗型**：游戏货币、道具
   - **非消耗型**：去广告、完整版
   - **自动续期订阅**：月卡、年卡

### 2. 创建StoreKit配置文件

**StoreKitConfiguration.swift**:
```swift
import StoreKit

@available(iOS 15.0, *)
class StoreKitWrapper: NSObject {

    private var productIds: Set<String> = []
    private var updateListenerTask: Task<Void, Error>?

    override init() {
        super.init()
        updateListenerTask = listenForTransactions()
    }

    // 监听交易更新
    private func listenForTransactions() -> Task<Void, Error> {
        return Task.detached {
            for await result in Transaction.updates {
                do {
                    let transaction = try self.checkVerified(result)
                    await self.processTransaction(transaction)
                    await transaction.finish()
                } catch {
                    print("Transaction verification failed: \(error)")
                }
            }
        }
    }

    private func checkVerified<T>(_ result: VerificationResult<T>) throws -> T {
        switch result {
        case .verified(let safe):
            return safe
        case .unverified:
            throw TransactionError.verificationFailed
        }
    }

    private func processTransaction(_ transaction: Transaction) async {
        guard let productID = transaction.productID else { return }

        switch transaction.productType {
        case .consumable:
            // 处理消耗型商品
            if transaction.revocationDate == nil {
                print("Consumable purchased: \(productID)")
            } else {
                print("Consumable revoked: \(productID)")
            }
        case .nonConsumable:
            // 处理非消耗型商品
            print("Non-consumable purchased: \(productID)")
        case .autoRenewable:
            // 处理订阅
            if transaction.revocationDate == nil {
                print("Subscription active: \(productID)")
            } else {
                print("Subscription expired: \(productID)")
            }
        default:
            break
        }
    }

    // 查询商品
    func queryProducts(productIds: [String]) async throws -> [Product] {
        self.productIds = Set(productIds)
        let products = try await Product.products(for: productIds)
        return products
    }

    // 购买商品
    func purchase(_ product: Product) async throws -> String? {
        let result = try await product.purchase()

        switch result {
        case .success(let verification):
            let transaction = try checkVerified(verification)
            return transaction.transactionID
        case .userCancelled:
            return nil
        case .pending:
            return nil
        @unknown default:
            return nil
        }
    }
}

enum TransactionError: Error {
    case verificationFailed
}
```

### 3. FFI桥接实现

**StoreKitFFI.m**:
```objective-c
#import <Foundation/Foundation.h>
#import <StoreKit/StoreKit.h>

int storekit_initialize_ffi(void) {
    if (@available(iOS 15.0, *)) {
        // Initialize StoreKit
        return 1; // Success
    }
    return 0; // Not supported
}

const char* storekit_query_products_ffi(const char* product_ids_json) {
    if (@available(iOS 15.0, *)) {
        NSString* productIdsStr = [NSString stringWithUTF8String:product_ids_json];
        NSData* data = [productIdsStr dataUsingEncoding:NSUTF8StringEncoding];
        NSArray* productIds = [NSJSONSerialization JSONObjectWithData:data
                                                             options:0
                                                               error:nil];

        // Query products using StoreKit 2.0
        // Return product details as JSON

        return strdup("{}"); // Placeholder
    }
    return NULL;
}

const char* storekit_purchase_ffi(const char* product_id) {
    if (@available(iOS 15.0, *)) {
        NSString* productIdStr = [NSString stringWithUTF8String:product_id];

        // Purchase product using StoreKit 2.0
        // Return purchase token

        return strdup(""); // Placeholder
    }
    return NULL;
}

const char* storekit_restore_ffi(void) {
    if (@available(iOS 15.0, *)) {
        // Restore purchases using StoreKit 2.0
        return strdup("[]"); // Placeholder
    }
    return NULL;
}

void storekit_free_string_ffi(const char* s) {
    if (s) {
        free((void*)s);
    }
}
```

## 脚本API使用

### JavaScript示例

```javascript
// 初始化应用内购买
iap_initialize();

// 查询商品信息
const products = JSON.parse(iap_query_products(["coin_pack_100", "no_ads", "premium_monthly"]));
console.log("商品列表：", products);

// 购买消耗型商品
const purchaseToken = iap_purchase("coin_pack_100");
if (purchaseToken) {
    console.log("购买成功，token：", purchaseToken);

    // 消耗购买
    iap_consume(purchaseToken);
    console.log("已消耗购买");
}

// 购买非消耗型商品
const noAdsToken = iap_purchase("no_ads");
if (noAdsToken) {
    console.log("已购买去广告");
}

// 恢复购买（非消耗型商品）
const purchases = JSON.parse(iap_restore());
console.log("已恢复购买：", purchases);

// 查询订阅状态
const subscription = JSON.parse(iap_query_subscription("premium_monthly"));
if (subscription && subscription.is_subscribed) {
    console.log("订阅有效，到期时间：", subscription.expire_time_ms);
}
```

### Lua示例

```lua
-- 初始化
iap_initialize()

-- 查询商品
local products_json = iap_query_products({"coin_pack_100", "no_ads"})
local products = json.decode(products_json)

for i, product in ipairs(products) do
    print(product.title, product.price)
end

-- 购买商品
local token = iap_purchase("coin_pack_100")
if token then
    print("购买成功：", token)
    iap_consume(token)
end

-- 恢复购买
local purchases_json = iap_restore()
local purchases = json.decode(purchases_json)

for i, purchase in ipairs(purchases) do
    print(purchase.product_id, purchase.is_acknowledged)
end
```

## 测试

### Android测试

1. **内部测试轨道**：
   - 上传APK到Google Play Console
   - 创建内部测试列表
   - 添加测试账号
   - 使用测试卡号进行测试

2. **测试商品**：
   - 在Google Play Console中激活测试商品
   - 使用测试账号登录设备
   - 购买测试商品（不会实际扣费）

3. **验证流程**：
   - 测试消耗型商品的购买和消耗
   - 测试非消耗型商品的购买和恢复
   - 测试订阅的购买和续费

### iOS测试

1. **Sandbox测试**：
   - 在App Store Connect中创建沙盒测试账号
   - 在设备设置中登录沙盒账号
   - 使用测试商品进行购买（不会实际扣费）

2. **StoreKit配置文件**：
   - Xcode → Scheme → Edit Scheme → Run → Options
   - 启用"StoreKit Configuration"
   - 选择或创建StoreKit配置文件

3. **验证流程**：
   - 测试消耗型商品购买
   - 测试非消耗型商品购买和恢复
   - 测试订阅的购买和续费
   - 测试收据验证

## 常见问题

**Android**: 商品查询失败
- 检查商品是否在Google Play Console中激活
- 确认应用签名与生产版本一致
- 等待商品激活（最多24小时）

**iOS**: 购买失败
- 检查App ID是否启用In-App Purchase
- 确认商品在App Store Connect中配置正确
- 验证沙盒测试账号状态

**跨平台**: 订阅续费问题
- Android: 处理BILLING_RESPONSE_CODE_FEATURE_NOT_SUPPORTED
- iOS: 使用StoreKit 2.0的自动续期订阅API
- 实现服务端订阅状态验证

## 性能建议

1. **商品缓存**: 缓存商品信息，避免频繁查询
2. **购买队列**: 实现购买队列，处理并发购买
3. **离线支持**: 缓存购买记录，支持离线验证
4. **重试机制**: 实现指数退避重试策略

## 安全建议

1. **服务端验证**: 所有购买必须在服务端验证收据
2. **加密通信**: 使用HTTPS传输购买数据
3. **签名验证**: 验证购买签名的有效性
4. **防篡改**: 检测和防止本地数据篡改

## 下一步

- 实现服务端收据验证
- 添加购买数据分析
- 实现促销和折扣系统
- 集成A/B测试框架

## 参考文档

- [Google Play Billing](https://developer.android.com/google/play/billing)
- [StoreKit 2.0](https://developer.apple.com/documentation/storekit/in-app_purchase)
- [App Store Connect](https://appstoreconnect.apple.com/)
