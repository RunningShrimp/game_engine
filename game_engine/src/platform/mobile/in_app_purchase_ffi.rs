//! # 应用内购买FFI绑定
//!
//! 提供与应用内购买服务的FFI接口绑定
//!
//! ## 支持的平台
//!
//! - **Android**: Google Play Billing Library
//! - **iOS**: StoreKit 2.0

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int};
use std::sync::{Arc, Mutex};

/// 商品类型
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProductType {
    Unknown = 0,
    /// 消耗型商品（游戏货币、道具等）
    Consumable = 1,
    /// 非消耗型商品（去广告、完整版等）
    NonConsumable = 2,
    /// 订阅型商品
    Subscription = 3,
}

/// 购买状态
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PurchaseStatus {
    Unknown = 0,
    /// 购买中
    Purchasing = 1,
    /// 购买成功
    Purchased = 2,
    /// 购买失败
    Failed = 3,
    /// 已恢复（非消耗型商品）
    Restored = 4,
}

/// 应用内购买FFI包装器（Android Google Play Billing）
#[cfg(target_os = "android")]
pub struct BillingFFI {
    initialized: bool,
    available: bool,
}

/// 应用内购买FFI包装器（iOS StoreKit）
#[cfg(target_os = "ios")]
pub struct StoreKitFFI {
    initialized: bool,
    available: bool,
}

// ===== Android Google Play Billing实现 =====

#[cfg(target_os = "android")]
impl BillingFFI {
    /// 创建新的Billing FFI包装器
    pub fn new() -> Self {
        Self {
            initialized: false,
            available: false,
        }
    }

    /// 初始化Google Play Billing
    pub fn initialize(&mut self) -> Result<(), String> {
        let result = unsafe { self.billing_initialize() };

        if result == 1 {
            self.initialized = true;
            self.available = true;
            tracing::info!("Google Play Billing initialized");
            Ok(())
        } else {
            Err("Failed to initialize Google Play Billing".to_string())
        }
    }

    /// 检查服务是否可用
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// 查询商品信息
    pub fn query_products(&self, product_ids: Vec<String>) -> Result<Vec<ProductInfo>, String> {
        if !self.is_available() {
            return Err("Billing service not available".to_string());
        }

        let products_json = unsafe {
            let product_ids_json = serde_json::to_string(&product_ids).unwrap_or_default();
            let product_ids_cstr = CString::new(product_ids_json.as_str()).unwrap();
            self.billing_query_products(product_ids_cstr.as_ptr())
        };

        if products_json.is_null() {
            return Err("Failed to query products".to_string());
        }

        let products_cstr = unsafe { CStr::from_ptr(products_json) };
        let products_json_str = products_cstr.to_string_lossy();

        // 释放内存
        unsafe { self.billing_free_string(products_json) };

        // 解析JSON
        serde_json::from_str(&products_json_str)
            .map_err(|_| "Failed to parse products JSON".to_string())
    }

    /// 购买商品
    pub fn purchase(&self, product_id: &str) -> Result<String, String> {
        if !self.is_available() {
            return Err("Billing service not available".to_string());
        }

        let product_cstring =
            CString::new(product_id).map_err(|_| "Invalid product ID".to_string())?;

        let purchase_token = unsafe {
            let token_ptr = self.billing_purchase(product_cstring.as_ptr());

            if token_ptr.is_null() {
                return Err("Purchase failed".to_string());
            }

            let token_cstr = CStr::from_ptr(token_ptr);
            let token_str = token_cstr.to_string_lossy().into_owned();

            // 释放token内存
            self.billing_free_string(token_ptr);

            token_str
        };

        tracing::info!("Purchase successful: {}", product_id);
        Ok(purchase_token)
    }

    /// 消耗购买（消耗型商品）
    pub fn consume(&self, purchase_token: &str) -> Result<(), String> {
        let token_cstring =
            CString::new(purchase_token).map_err(|_| "Invalid purchase token".to_string())?;

        let result = unsafe { self.billing_consume(token_cstring.as_ptr()) };

        if result == 1 {
            tracing::info!("Purchase consumed: {}", purchase_token);
            Ok(())
        } else {
            Err("Failed to consume purchase".to_string())
        }
    }

    /// 恢复购买（非消耗型商品）
    pub fn restore_purchases(&self) -> Result<Vec<PurchaseInfo>, String> {
        let purchases_json = unsafe { self.billing_restore() };

        if purchases_json.is_null() {
            return Ok(Vec::new());
        }

        let purchases_cstr = unsafe { CStr::from_ptr(purchases_json) };
        let purchases_json_str = purchases_cstr.to_string_lossy();

        // 释放内存
        unsafe { self.billing_free_string(purchases_json) };

        serde_json::from_str(&purchases_json_str)
            .map_err(|_| "Failed to parse purchases JSON".to_string())
    }

    /// 订阅状态查询
    pub fn query_subscription(&self, product_id: &str) -> Result<Option<SubscriptionInfo>, String> {
        let product_cstring =
            CString::new(product_id).map_err(|_| "Invalid product ID".to_string())?;

        let sub_json = unsafe { self.billing_query_subscription(product_cstring.as_ptr()) };

        if sub_json.is_null() {
            return Ok(None);
        }

        let sub_cstr = unsafe { CStr::from_ptr(sub_json) };
        let sub_json_str = sub_cstr.to_string_lossy();

        // 释放内存
        unsafe { self.billing_free_string(sub_json) };

        let sub: SubscriptionInfo = serde_json::from_str(&sub_json_str)
            .map_err(|_| "Failed to parse subscription JSON".to_string())?;

        Ok(Some(sub))
    }

    // ===== FFI方法声明 =====

    unsafe fn billing_initialize(&self) -> c_int {
        extern "C" {
            fn billing_initialize_ffi() -> c_int;
        }
        billing_initialize_ffi()
    }

    unsafe fn billing_query_products(&self, product_ids_json: *const c_char) -> *const c_char {
        extern "C" {
            fn billing_query_products_ffi(product_ids: *const c_char) -> *const c_char;
        }
        billing_query_products_ffi(product_ids_json)
    }

    unsafe fn billing_purchase(&self, product_id: *const c_char) -> *const c_char {
        extern "C" {
            fn billing_purchase_ffi(product_id: *const c_char) -> *const c_char;
        }
        billing_purchase_ffi(product_id)
    }

    unsafe fn billing_consume(&self, purchase_token: *const c_char) -> c_int {
        extern "C" {
            fn billing_consume_ffi(purchase_token: *const c_char) -> c_int;
        }
        billing_consume_ffi(purchase_token)
    }

    unsafe fn billing_restore(&self) -> *const c_char {
        extern "C" {
            fn billing_restore_ffi() -> *const c_char;
        }
        billing_restore_ffi()
    }

    unsafe fn billing_query_subscription(&self, product_id: *const c_char) -> *const c_char {
        extern "C" {
            fn billing_query_subscription_ffi(product_id: *const c_char) -> *const c_char;
        }
        billing_query_subscription_ffi(product_id)
    }

    unsafe fn billing_free_string(&self, s: *const c_char) {
        extern "C" {
            fn billing_free_string_ffi(s: *const c_char);
        }
        billing_free_string_ffi(s)
    }
}

#[cfg(target_os = "android")]
unsafe impl Send for BillingFFI {}
unsafe impl Sync for BillingFFI {}

#[cfg(target_os = "android")]
impl Default for BillingFFI {
    fn default() -> Self {
        Self::new()
    }
}

// ===== iOS StoreKit实现 =====

#[cfg(target_os = "ios")]
impl StoreKitFFI {
    /// 创建新的StoreKit FFI包装器
    pub fn new() -> Self {
        Self {
            initialized: false,
            available: false,
        }
    }

    /// 初始化StoreKit
    pub fn initialize(&mut self) -> Result<(), String> {
        let result = unsafe { self.storekit_initialize() };

        if result == 1 {
            self.initialized = true;
            self.available = true;
            tracing::info!("StoreKit initialized");
            Ok(())
        } else {
            Err("Failed to initialize StoreKit".to_string())
        }
    }

    /// 检查服务是否可用
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// 查询商品信息
    pub fn query_products(&self, product_ids: Vec<String>) -> Result<Vec<ProductInfo>, String> {
        if !self.is_available() {
            return Err("StoreKit not available".to_string());
        }

        let products_json = unsafe {
            let product_ids_json = serde_json::to_string(&product_ids).unwrap_or_default();
            let product_ids_cstr = CString::new(product_ids_json.as_str()).unwrap();
            self.storekit_query_products(product_ids_cstr.as_ptr())
        };

        if products_json.is_null() {
            return Err("Failed to query products".to_string());
        }

        let products_cstr = unsafe { CStr::from_ptr(products_json) };
        let products_json_str = products_cstr.to_string_lossy();

        // 释放内存
        unsafe { self.storekit_free_string(products_json) };

        serde_json::from_str(&products_json_str)
            .map_err(|_| "Failed to parse products JSON".to_string())
    }

    /// 购买商品
    pub fn purchase(&self, product_id: &str) -> Result<String, String> {
        if !self.is_available() {
            return Err("StoreKit not available".to_string());
        }

        let product_cstring =
            CString::new(product_id).map_err(|_| "Invalid product ID".to_string())?;

        let purchase_token = unsafe {
            let token_ptr = self.storekit_purchase(product_cstring.as_ptr());

            if token_ptr.is_null() {
                return Err("Purchase failed".to_string());
            }

            let token_cstr = CStr::from_ptr(token_ptr);
            let token_str = token_cstr.to_string_lossy().into_owned();

            // 释放token内存
            self.storekit_free_string(token_ptr);

            token_str
        };

        tracing::info!("Purchase successful: {}", product_id);
        Ok(purchase_token)
    }

    /// 恢复购买（非消耗型商品）
    pub fn restore_purchases(&self) -> Result<Vec<PurchaseInfo>, String> {
        let purchases_json = unsafe { self.storekit_restore() };

        if purchases_json.is_null() {
            return Ok(Vec::new());
        }

        let purchases_cstr = unsafe { CStr::from_ptr(purchases_json) };
        let purchases_json_str = purchases_cstr.to_string_lossy();

        // 释放内存
        unsafe { self.storekit_free_string(purchases_json) };

        serde_json::from_str(&purchases_json_str)
            .map_err(|_| "Failed to parse purchases JSON".to_string())
    }

    /// 订阅状态查询
    pub fn query_subscription(&self, product_id: &str) -> Result<Option<SubscriptionInfo>, String> {
        let product_cstring =
            CString::new(product_id).map_err(|_| "Invalid product ID".to_string())?;

        let sub_json = unsafe { self.storekit_query_subscription(product_cstring.as_ptr()) };

        if sub_json.is_null() {
            return Ok(None);
        }

        let sub_cstr = unsafe { CStr::from_ptr(sub_json) };
        let sub_json_str = sub_cstr.to_string_lossy();

        // 释放内存
        unsafe { self.storekit_free_string(sub_json) };

        let sub: SubscriptionInfo = serde_json::from_str(&sub_json_str)
            .map_err(|_| "Failed to parse subscription JSON".to_string())?;

        Ok(Some(sub))
    }

    /// 获取订阅状态（群组订阅）
    pub fn get_subscription_status(&self, group_id: &str) -> Result<Vec<SubscriptionInfo>, String> {
        let group_cstring = CString::new(group_id).map_err(|_| "Invalid group ID".to_string())?;

        let status_json = unsafe { self.storekit_get_subscription_status(group_cstring.as_ptr()) };

        if status_json.is_null() {
            return Err("Failed to get subscription status".to_string());
        }

        let status_cstr = unsafe { CStr::from_ptr(status_json) };
        let status_json_str = status_cstr.to_string_lossy();

        // 释放内存
        unsafe { self.storekit_free_string(status_json) };

        serde_json::from_str(&status_json_str)
            .map_err(|_| "Failed to parse subscription status JSON".to_string())
    }

    // ===== FFI方法声明 =====

    unsafe fn storekit_initialize(&self) -> c_int {
        extern "C" {
            fn storekit_initialize_ffi() -> c_int;
        }
        storekit_initialize_ffi()
    }

    unsafe fn storekit_query_products(&self, product_ids_json: *const c_char) -> *const c_char {
        extern "C" {
            fn storekit_query_products_ffi(product_ids: *const c_char) -> *const c_char;
        }
        storekit_query_products_ffi(product_ids_json)
    }

    unsafe fn storekit_purchase(&self, product_id: *const c_char) -> *const c_char {
        extern "C" {
            fn storekit_purchase_ffi(product_id: *const c_char) -> *const c_char;
        }
        storekit_purchase_ffi(product_id)
    }

    unsafe fn storekit_restore(&self) -> *const c_char {
        extern "C" {
            fn storekit_restore_ffi() -> *const c_char;
        }
        storekit_restore_ffi()
    }

    unsafe fn storekit_query_subscription(&self, product_id: *const c_char) -> *const c_char {
        extern "C" {
            fn storekit_query_subscription_ffi(product_id: *const c_char) -> *const c_char;
        }
        storekit_query_subscription_ffi(product_id)
    }

    unsafe fn storekit_get_subscription_status(&self, group_id: *const c_char) -> *const c_char {
        extern "C" {
            fn storekit_get_subscription_status_ffi(group_id: *const c_char) -> *const c_char;
        }
        storekit_get_subscription_status_ffi(group_id)
    }

    unsafe fn storekit_free_string(&self, s: *const c_char) {
        extern "C" {
            fn storekit_free_string_ffi(s: *const c_char);
        }
        storekit_free_string_ffi(s)
    }
}

#[cfg(target_os = "ios")]
unsafe impl Send for StoreKitFFI {}
unsafe impl Sync for StoreKitFFI {}

#[cfg(target_os = "ios")]
impl Default for StoreKitFFI {
    fn default() -> Self {
        Self::new()
    }
}

// ===== 数据结构 =====

/// 商品信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProductInfo {
    pub product_id: String,
    pub title: String,
    pub description: String,
    pub price: String,
    pub price_amount_micros: i64,
    pub currency_code: String,
    pub product_type: ProductType,
}

/// 购买信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PurchaseInfo {
    pub product_id: String,
    pub purchase_token: String,
    pub purchase_time: i64,
    pub quantity: i32,
    pub is_acknowledged: bool,
}

/// 订阅信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubscriptionInfo {
    pub product_id: String,
    pub is_subscribed: bool,
    pub will_renew: bool,
    pub expire_time_ms: Option<i64>,
    pub group_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_type() {
        assert_eq!(ProductType::Consumable, ProductType::Consumable);
        assert_eq!(ProductType::Subscription, ProductType::Subscription);
    }
}
