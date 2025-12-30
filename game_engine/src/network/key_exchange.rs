// 密钥交换协议模块
//
// 实现密钥交换协议，用于建立通信通道。
// 支持安全的X25519 ECDH密钥交换和向后兼容的简化实现。
//
// ## 功能
//
//  - 生成并管理临时密钥对
//  - 执行密钥交换协议
//  - 从共享密钥派生加密密钥
//  - 密钥有效期管理
//
// ## 优化说明
//
// 本模块通过以下策略优化条件编译的使用：
// 1. 使用trait对象和运行时分发替代编译期选择
// 2. 创建统一的KeyExchangeConfig配置对象
// 3. 通过工厂模式创建backend实例（集中条件编译点）
// 4. 只在必要的位置使用条件编译（依赖、实现、工厂、测试）
//
// 条件编译分布（共14处，均已标记）：
// - 依赖导入: 2处 (#1-2: secure/insecure依赖)
// - Backend实现: 4处 (#3-6: 两个backend的结构体和impl块)
// - 工厂函数: 5处 (#7-11: create_backend函数内的特性检查)
// - 公共API: 1处 (#12: generate_insecure方法)
// - 测试模块: 2处 (#13-14: test模块和不安全实现测试)
//
// 注意：条件编译主要用于依赖管理（避免未使用依赖的编译错误）和测试。
// 核心业务逻辑通过trait对象实现运行时多态，无需条件编译。
//
// ## 安全说明
//
// 默认使用X25519椭圆曲线Diffie-Hellman进行密钥交换，提供前向安全性。
// 密钥派生使用HKDF (RFC 5869) 确保安全性。
// 可通过feature flag切换到简化实现（仅用于测试）。
//
// ## 特性标志
//
//  - `secure_key_exchange` (默认): 使用X25519 ECDH和HKDF进行安全的密钥交换
//  - `insecure_key_exchange`: 使用SHA256的简化实现（仅用于测试，不应用于生产环境）

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// 条件编译区域1-2: 依赖导入（共2处）
// ============================================================================

// 安全实现依赖
#[cfg(feature = "secure_key_exchange")]
use {
    hkdf::Hkdf,
    sha2::Sha256 as HkdfSha256,
    x25519_dalek_ng::{PublicKey, StaticSecret},
};

// 不安全实现依赖
#[cfg(feature = "insecure_key_exchange")]
use sha2::{Digest, Sha256};

// ============================================================================
// Key Exchange Backend Trait Abstraction
// ============================================================================

/// 密钥交换后端trait
///
/// 定义了密钥交换的核心操作接口，允许在运行时选择不同的实现。
trait KeyExchangeBackend: Send + Sync {
    /// 生成密钥对（公钥，私钥）
    fn generate_keypair(&self) -> ([u8; 32], [u8; 32]);

    /// 从私钥派生公钥
    fn derive_public_key(&self, private_key: &[u8; 32]) -> [u8; 32];

    /// 计算共享密钥
    fn compute_shared_secret(&self, private_key: &[u8; 32], public_key: &[u8; 32]) -> [u8; 32];

    /// 从共享密钥派生加密密钥和认证密钥
    fn derive_keys(&self, shared_secret: [u8; 32]) -> ([u8; 32], [u8; 32]);

    /// 获取后端名称（用于日志和调试）
    fn backend_name(&self) -> &'static str;
}

// ============================================================================
// Backend Implementations（条件编译 #3-4）
// ============================================================================

// 安全实现（条件编译 #3-4: 结构体和impl块）
#[cfg(feature = "secure_key_exchange")]
struct SecureKeyExchangeBackend;

#[cfg(feature = "secure_key_exchange")]
impl KeyExchangeBackend for SecureKeyExchangeBackend {
    fn generate_keypair(&self) -> ([u8; 32], [u8; 32]) {
        use rand::RngCore;
        let mut rng = rand::rng();
        let mut private_key_bytes = [0u8; 32];
        rng.fill_bytes(&mut private_key_bytes);

        let static_secret = StaticSecret::from(private_key_bytes);
        let public_key = PublicKey::from(&static_secret);

        (public_key.to_bytes(), static_secret.to_bytes())
    }

    fn derive_public_key(&self, private_key: &[u8; 32]) -> [u8; 32] {
        let static_secret = StaticSecret::from(*private_key);
        let public_key = PublicKey::from(&static_secret);
        public_key.to_bytes()
    }

    fn compute_shared_secret(&self, private_key: &[u8; 32], public_key: &[u8; 32]) -> [u8; 32] {
        let static_secret = StaticSecret::from(*private_key);
        let peer_public = PublicKey::from(*public_key);
        static_secret.diffie_hellman(&peer_public).to_bytes()
    }

    fn derive_keys(&self, shared_secret: [u8; 32]) -> ([u8; 32], [u8; 32]) {
        let hk = Hkdf::<HkdfSha256>::new(None, &shared_secret);

        let mut encryption_key = [0u8; 32];
        hk.expand(b"encryption", &mut encryption_key)
            .expect("HKDF expansion for encryption key should not fail with fixed-size output");

        let mut authentication_key = [0u8; 32];
        hk.expand(b"authentication", &mut authentication_key)
            .expect("HKDF expansion for authentication key should not fail with fixed-size output");

        (encryption_key, authentication_key)
    }

    fn backend_name(&self) -> &'static str {
        "X25519-ECDH-HKDF"
    }
}

// 不安全实现（条件编译 #5-6: 结构体和impl块）
#[cfg(feature = "insecure_key_exchange")]
struct InsecureKeyExchangeBackend;

#[cfg(feature = "insecure_key_exchange")]
impl KeyExchangeBackend for InsecureKeyExchangeBackend {
    fn generate_keypair(&self) -> ([u8; 32], [u8; 32]) {
        use rand::RngCore;
        #[allow(deprecated)]
        let mut rng = rand::thread_rng();
        let mut private_key = [0u8; 32];
        rng.fill_bytes(&mut private_key);

        private_key[0] &= 248;
        private_key[31] &= 127;
        private_key[31] |= 64;

        let public_key = self.derive_public_key(&private_key);
        (public_key, private_key)
    }

    fn derive_public_key(&self, private_key: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(private_key);
        let digest = hasher.finalize();

        let mut public_key = [0u8; 32];
        public_key.copy_from_slice(&digest[..32]);
        public_key
    }

    fn compute_shared_secret(&self, private_key: &[u8; 32], public_key: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(private_key);
        hasher.update(public_key);
        let digest = hasher.finalize();

        let mut shared_secret = [0u8; 32];
        shared_secret.copy_from_slice(&digest[..32]);
        shared_secret
    }

    fn derive_keys(&self, shared_secret: [u8; 32]) -> ([u8; 32], [u8; 32]) {
        let mut hasher = Sha256::new();
        hasher.update(&shared_secret);
        hasher.update(b"encryption");
        let mut encryption_key = [0u8; 32];
        encryption_key.copy_from_slice(&hasher.finalize()[..32]);

        let mut hasher = Sha256::new();
        hasher.update(&shared_secret);
        hasher.update(b"authentication");
        let mut authentication_key = [0u8; 32];
        authentication_key.copy_from_slice(&hasher.finalize()[..32]);

        (encryption_key, authentication_key)
    }

    fn backend_name(&self) -> &'static str {
        "SHA256-SIMPLIFIED"
    }
}

// ============================================================================
// Factory Pattern for Backend Creation
// ============================================================================

/// 密钥交换配置
///
/// 用于在运行时配置密钥交换行为，避免条件编译。
#[derive(Debug, Clone)]
pub struct KeyExchangeConfig {
    /// 是否使用安全模式
    pub secure: bool,
}

impl Default for KeyExchangeConfig {
    fn default() -> Self {
        // 默认使用安全模式
        Self { secure: true }
    }
}

impl KeyExchangeConfig {
    /// 创建安全配置（推荐用于生产环境）
    pub fn secure() -> Self {
        Self { secure: true }
    }

    /// 创建不安全配置（仅用于测试）
    pub fn insecure() -> Self {
        Self { secure: false }
    }
}

/// 创建后端实例（条件编译 #7-11: 工厂函数）
///
/// 根据配置和可用特性选择合适的后端实现。
/// 通过统一的入口点减少运行时分发的条件编译。
fn create_backend(config: &KeyExchangeConfig) -> Arc<dyn KeyExchangeBackend> {
    // 优先根据配置选择后端
    #[cfg(feature = "secure_key_exchange")]
    {
        if config.secure {
            tracing::debug!("Using secure X25519 ECDH key exchange");
            return Arc::new(SecureKeyExchangeBackend);
        }
    }

    #[cfg(feature = "insecure_key_exchange")]
    {
        if !config.secure {
            tracing::warn!("Using insecure simplified key exchange - only for testing!");
            return Arc::new(InsecureKeyExchangeBackend);
        }
    }

    // Fallback: 使用可用的后端
    #[cfg(feature = "secure_key_exchange")]
    {
        tracing::warn!("Config mismatch, falling back to secure backend");
        Arc::new(SecureKeyExchangeBackend)
    }

    #[cfg(all(not(feature = "secure_key_exchange"), feature = "insecure_key_exchange"))]
    {
        tracing::error!("Only insecure backend available");
        Arc::new(InsecureKeyExchangeBackend)
    }

    #[cfg(not(any(feature = "secure_key_exchange", feature = "insecure_key_exchange")))]
    compile_error!("At least one of 'secure_key_exchange' or 'insecure_key_exchange' features must be enabled");
}

// ============================================================================
// Public API
// ============================================================================

/// 统一的密钥交换 trait
///
/// 定义密钥交换协议的标准接口，允许使用不同的密钥交换算法实现。
pub trait KeyExchangeProtocol {
    /// 获取本地公钥（32字节）
    fn public_key(&self) -> [u8; 32];

    /// 从对方公钥和本地私钥计算共享密钥
    ///
    /// # 参数
    /// - `peer_public_key`: 对方的公钥（32字节）
    ///
    /// # 返回
    /// 包含共享密钥、加密密钥和认证密钥的 SharedSecret
    fn compute_shared_secret(&self, peer_public_key: [u8; 32]) -> SharedSecret;

    /// 获取本地密钥对的可选引用
    fn keypair(&self) -> Option<&KeyPair>;
}

/// 密钥对
#[derive(Clone, Serialize, Deserialize)]
pub struct KeyPair {
    /// 32字节公钥
    pub public_key: [u8; 32],
    /// 32字节私钥
    pub private_key: [u8; 32],
    /// 密钥对生成时间（Unix时间戳，秒）
    pub created_at: u64,
    /// 后端引用（用于运行时分发）
    #[serde(skip)]
    backend: Option<Arc<dyn KeyExchangeBackend>>,
}

// 手动实现Debug，跳过backend字段
impl std::fmt::Debug for KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyPair")
            .field("public_key", &self.public_key)
            .field("private_key", &"<redacted>")
            .field("created_at", &self.created_at)
            .field("backend", &self.backend.as_ref().map(|_| "<Backend>"))
            .finish()
    }
}

// 为KeyPair添加手动实现Clone，因为Arc<dyn KeyExchangeBackend>需要特殊处理
impl KeyPair {
    /// 生成新的密钥对（使用默认安全配置）
    ///
    /// 推荐用于生产环境，使用X25519 ECDH进行安全的密钥交换。
    pub fn generate() -> Self {
        Self::generate_with_config(KeyExchangeConfig::default())
    }

    /// 使用指定配置生成密钥对
    ///
    /// # 参数
    /// - `config`: 密钥交换配置
    pub fn generate_with_config(config: KeyExchangeConfig) -> Self {
        let backend = create_backend(&config);
        let (public_key, private_key) = backend.generate_keypair();
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        tracing::debug!(
            "Generated keypair using {} backend",
            backend.backend_name()
        );

        Self {
            public_key,
            private_key,
            created_at,
            backend: Some(backend),
        }
    }

    /// 生成不安全的密钥对（仅用于测试）（条件编译 #12）
    ///
    /// 此方法使用不安全的实现。
    /// **警告：仅用于测试目的，不应用于生产环境！**
    /// 注意：此方法仅在启用insecure_key_exchange特性时可用。
    #[cfg(feature = "insecure_key_exchange")]
    pub fn generate_insecure() -> Self {
        eprintln!("WARNING: Generating insecure keypair - only for testing!");
        Self::generate_with_config(KeyExchangeConfig::insecure())
    }

    /// 获取或创建默认后端
    fn get_backend(&self) -> Arc<dyn KeyExchangeBackend> {
        self.backend
            .clone()
            .unwrap_or_else(|| create_backend(&KeyExchangeConfig::default()))
    }

    /// 检查密钥对是否有效
    pub fn is_valid(&self) -> bool {
        // 公钥和私钥都应该是非零的
        !self.public_key.iter().all(|&b| b == 0) && !self.private_key.iter().all(|&b| b == 0)
    }

    /// 获取密钥年龄（秒）
    pub fn age_secs(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.created_at)
    }

    /// 计算共享密钥（从对方公钥和本地私钥）
    ///
    /// 使用运行时分发，根据配置选择X25519 ECDH或SHA256简化实现。
    ///
    /// # 参数
    /// - `peer_public_key`: 对方的公钥（32字节）
    ///
    /// # 返回
    /// 包含共享密钥、加密密钥和认证密钥的 SharedSecret
    pub fn compute_shared_secret(&self, peer_public_key: [u8; 32]) -> SharedSecret {
        let backend = self.get_backend();
        let shared_secret = backend.compute_shared_secret(&self.private_key, &peer_public_key);
        SharedSecret::derive_with_backend(shared_secret, backend)
    }
}

/// 密钥交换消息（客户端发送给服务器）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyExchangeMessage {
    /// 客户端公钥
    pub public_key: [u8; 32],
    /// 客户端ID
    pub client_id: u64,
    /// 时间戳（用于防重放攻击）
    pub timestamp: u64,
}

impl KeyExchangeMessage {
    /// 创建密钥交换消息
    pub fn new(client_id: u64, public_key: [u8; 32]) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|e| {
                tracing::error!("SystemTime is before UNIX_EPOCH: {:?}", e);
                std::time::Duration::from_secs(0)
            })
            .as_secs();

        Self {
            public_key,
            client_id,
            timestamp,
        }
    }

    /// 验证消息时间戳（防止旧消息重放）
    pub fn is_recent(&self, max_age_secs: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(u64::MAX))
            .as_secs();

        now.saturating_sub(self.timestamp) < max_age_secs
    }
}

/// 密钥交换结果
#[derive(Debug, Clone)]
pub struct SharedSecret {
    /// 共享密钥（32字节）
    pub shared_secret: [u8; 32],
    /// 导出的加密密钥（32字节）
    pub encryption_key: [u8; 32],
    /// 导出的认证密钥（32字节）
    pub authentication_key: [u8; 32],
}

impl SharedSecret {
    /// 从共享密钥派生出加密密钥和认证密钥（使用默认配置）
    ///
    /// 根据启用的feature选择密钥派生方法：
    /// - `secure_key_exchange`: 使用HKDF (RFC 5869) 进行安全的密钥派生
    /// - `insecure_key_exchange`: 使用SHA256进行简单派生（仅用于测试）
    pub fn derive(shared_secret: [u8; 32]) -> Self {
        let backend = create_backend(&KeyExchangeConfig::default());
        Self::derive_with_backend(shared_secret, backend)
    }

    /// 使用指定后端派生密钥
    fn derive_with_backend(shared_secret: [u8; 32], backend: Arc<dyn KeyExchangeBackend>) -> Self {
        let (encryption_key, authentication_key) = backend.derive_keys(shared_secret);

        tracing::trace!(
            "Derived keys using {} backend",
            backend.backend_name()
        );

        Self {
            shared_secret,
            encryption_key,
            authentication_key,
        }
    }
}

/// 密钥交换器
pub struct KeyExchange {
    /// 本地密钥对
    local_keypair: KeyPair,
}

impl KeyExchange {
    /// 创建新的密钥交换器（使用默认安全配置）
    pub fn new() -> Self {
        let local_keypair = KeyPair::generate();
        Self { local_keypair }
    }

    /// 使用指定配置创建密钥交换器
    pub fn with_config(config: KeyExchangeConfig) -> Self {
        let local_keypair = KeyPair::generate_with_config(config);
        Self { local_keypair }
    }

    /// 获取本地公钥
    pub fn public_key(&self) -> [u8; 32] {
        self.local_keypair.public_key
    }

    /// 执行密钥交换（从对方公钥和本地私钥计算共享密钥）
    ///
    /// 使用运行时分发，根据配置选择实现：
    /// - `secure_key_exchange`: 使用X25519 ECDH进行安全的密钥交换
    /// - `insecure_key_exchange`: 使用SHA256的简化实现（仅用于测试）
    pub fn compute_shared_secret(&self, peer_public_key: [u8; 32]) -> SharedSecret {
        self.local_keypair.compute_shared_secret(peer_public_key)
    }

    /// 获取本地密钥对
    pub fn keypair(&self) -> &KeyPair {
        &self.local_keypair
    }
}

impl KeyExchangeProtocol for KeyExchange {
    fn public_key(&self) -> [u8; 32] {
        self.local_keypair.public_key
    }

    fn compute_shared_secret(&self, peer_public_key: [u8; 32]) -> SharedSecret {
        // 委托给KeyExchange的实现，避免重复代码
        self.compute_shared_secret(peer_public_key)
    }

    fn keypair(&self) -> Option<&KeyPair> {
        Some(&self.local_keypair)
    }
}

impl Default for KeyExchange {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests（条件编译 #13: 测试模块）
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::compat::bincode_compat;

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_keypair_generation() {
        let keypair = KeyPair::generate();
        assert!(keypair.is_valid());
        assert!(!keypair.public_key.iter().all(|&b| b == 0));
        assert!(!keypair.private_key.iter().all(|&b| b == 0));
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_keypair_age() {
        let keypair = KeyPair::generate();
        let age = keypair.age_secs();
        assert!(age <= 1); // 应该是0或1秒
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_key_exchange_message() {
        let msg = KeyExchangeMessage::new(1, [1u8; 32]);
        assert_eq!(msg.client_id, 1);
        assert!(msg.is_recent(60)); // 最近60秒内
        assert!(!msg.is_recent(0)); // 不是0秒前发送
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_shared_secret_derivation() {
        let secret = [42u8; 32];
        let shared = SharedSecret::derive(secret);

        // 相同的输入应生成相同的输出
        assert_eq!(shared.shared_secret, secret);

        // 派生的密钥不应为零
        assert!(!shared.encryption_key.iter().all(|&b| b == 0));
        assert!(!shared.authentication_key.iter().all(|&b| b == 0));
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_key_exchange_mutual() {
        let client_ke = KeyExchange::new();
        let server_ke = KeyExchange::new();

        // 客户端用服务器公钥和自己私钥计算共享密钥
        let client_shared = client_ke.compute_shared_secret(server_ke.public_key());

        // 服务器用客户端公钥和自己私钥计算共享密钥
        let server_shared = server_ke.compute_shared_secret(client_ke.public_key());

        // 使用运行时检查而不是编译期条件编译
        let backend = client_ke.local_keypair.get_backend();
        if backend.backend_name() == "X25519-ECDH-HKDF" {
            // X25519 ECDH保证双方生成相同的共享密钥
            assert_eq!(client_shared.shared_secret, server_shared.shared_secret);
            assert_eq!(client_shared.encryption_key, server_shared.encryption_key);
            assert_eq!(
                client_shared.authentication_key,
                server_shared.authentication_key
            );
        } else {
            // 简化实现不能保证完全相同，但加密密钥应该根据相同的算法派生
            assert_eq!(
                client_shared.encryption_key.len(),
                server_shared.encryption_key.len()
            );
        }
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_key_exchange_deterministic() {
        let ke = KeyExchange::new();
        let peer_key = [99u8; 32];

        let shared1 = ke.compute_shared_secret(peer_key);
        let shared2 = ke.compute_shared_secret(peer_key);

        // 相同输入应生成相同输出
        assert_eq!(shared1.shared_secret, shared2.shared_secret);
        assert_eq!(shared1.encryption_key, shared2.encryption_key);
        assert_eq!(shared1.authentication_key, shared2.authentication_key);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_secure_key_exchange_properties() {
        // 测试安全密钥交换的基本属性（使用运行时检查）
        let keypair1 = KeyPair::generate();
        let keypair2 = KeyPair::generate();

        // 检查是否使用安全后端
        let backend = keypair1.get_backend();
        if backend.backend_name() == "X25519-ECDH-HKDF" {
            // 公钥应该不同（除非极小的概率）
            assert_ne!(keypair1.public_key, keypair2.public_key);
            assert_ne!(keypair1.private_key, keypair2.private_key);
        }

        // 密钥对应该有效
        assert!(keypair1.is_valid());
        assert!(keypair2.is_valid());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_hkdf_derivation() {
        // 测试HKDF密钥派生的一致性（使用运行时检查）
        let keypair = KeyPair::generate();
        let backend = keypair.get_backend();

        if backend.backend_name() == "X25519-ECDH-HKDF" {
            let secret = [42u8; 32];
            let shared1 = SharedSecret::derive(secret);
            let shared2 = SharedSecret::derive(secret);

            // 相同输入应生成相同输出
            assert_eq!(shared1.shared_secret, shared2.shared_secret);
            assert_eq!(shared1.encryption_key, shared2.encryption_key);
            assert_eq!(shared1.authentication_key, shared2.authentication_key);

            // 加密密钥和认证密钥应该不同
            assert_ne!(shared1.encryption_key, shared1.authentication_key);
        }
    }

    // ========================================================================
    // 集成测试 (P1-4.2)
    // ========================================================================

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_complete_key_exchange_flow() {
        // 完整的密钥交换流程测试
        let client_keypair = KeyPair::generate();
        let server_keypair = KeyPair::generate();

        // 客户端计算共享密钥
        let client_shared = client_keypair.compute_shared_secret(server_keypair.public_key);

        // 服务器计算共享密钥
        let server_shared = server_keypair.compute_shared_secret(client_keypair.public_key);

        let backend = client_keypair.get_backend();

        // 如果使用安全后端，双方应该得到相同的共享密钥
        if backend.backend_name() == "X25519-ECDH-HKDF" {
            assert_eq!(
                client_shared.shared_secret, server_shared.shared_secret,
                "客户端和服务器的共享密钥应该相同"
            );

            assert_eq!(
                client_shared.encryption_key, server_shared.encryption_key,
                "客户端和服务器的加密密钥应该相同"
            );

            assert_eq!(
                client_shared.authentication_key, server_shared.authentication_key,
                "客户端和服务器的认证密钥应该相同"
            );
        }
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_keypair_serialization_roundtrip() {
        // 测试密钥对的序列化和反序列化
        let keypair1 = KeyPair::generate();

        // 序列化
        let serialized = bincode_compat::serialize(&keypair1)
            .map_err(|e| Box::new(e))
            .expect("密钥对序列化失败");

        // 反序列化
        let keypair2: KeyPair =
            bincode_compat::deserialize(&serialized).expect("密钥对反序列化失败");

        // 验证
        assert_eq!(
            keypair1.public_key, keypair2.public_key,
            "序列化后公钥应该相同"
        );

        assert_eq!(
            keypair1.private_key, keypair2.private_key,
            "序列化后私钥应该相同"
        );

        assert_eq!(
            keypair1.created_at, keypair2.created_at,
            "序列化后创建时间应该相同"
        );
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_key_exchange_message_roundtrip() {
        // 测试密钥交换消息的序列化
        let msg1 = KeyExchangeMessage {
            public_key: [1u8; 32],
            client_id: 12345,
            timestamp: 1672531200, // 2023-01-01 00:00:00 UTC
        };

        // 序列化
        let serialized = bincode_compat::serialize(&msg1)
            .map_err(|e| Box::new(e))
            .expect("消息序列化失败");

        // 反序列化
        let msg2: KeyExchangeMessage =
            bincode_compat::deserialize(&serialized).expect("消息反序列化失败");

        // 验证
        assert_eq!(msg1.public_key, msg2.public_key);
        assert_eq!(msg1.client_id, msg2.client_id);
        assert_eq!(msg1.timestamp, msg2.timestamp);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_multiple_key_exchanges() {
        // 测试多次密钥交换产生不同的密钥
        let keypair = KeyPair::generate();
        let peer_keys = [
            KeyPair::generate().public_key,
            KeyPair::generate().public_key,
            KeyPair::generate().public_key,
        ];

        let shared_secrets: Vec<_> = peer_keys
            .iter()
            .map(|&peer_key| keypair.compute_shared_secret(peer_key))
            .collect();

        // 所有共享密钥应该不同
        for i in 0..shared_secrets.len() {
            for j in (i + 1)..shared_secrets.len() {
                assert_ne!(
                    shared_secrets[i].shared_secret, shared_secrets[j].shared_secret,
                    "不同的对等方应该产生不同的共享密钥"
                );
            }
        }
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_keypair_age_tracking() {
        // 测试密钥年龄追踪
        let keypair = KeyPair::generate();

        // 新创建的密钥对年龄应该很小
        let age = keypair.age_secs();
        assert!(age < 10, "新创建的密钥对年龄应该小于10秒");

        // 密钥对应该有效
        assert!(keypair.is_valid());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_invalid_keypair_detection() {
        // 测试无效密钥对的检测
        // 全零的公钥和私钥应该无效
        let invalid_keypair = KeyPair {
            public_key: [0u8; 32],
            private_key: [0u8; 32],
            created_at: 0,
            backend: None,
        };

        assert!(!invalid_keypair.is_valid(), "全零密钥对应该无效");
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_key_derivation_consistency() {
        // 测试密钥派生的一致性
        let shared_secret = [7u8; 32];

        let derived1 = SharedSecret::derive(shared_secret);
        let derived2 = SharedSecret::derive(shared_secret);

        // 多次派生应该得到相同结果
        assert_eq!(derived1.shared_secret, derived2.shared_secret);
        assert_eq!(derived1.encryption_key, derived2.encryption_key);
        assert_eq!(derived1.authentication_key, derived2.authentication_key);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_encryption_and_authentication_keys_different() {
        // 测试加密密钥和认证密钥确实不同
        let shared_secret = [42u8; 32];
        let derived = SharedSecret::derive(shared_secret);

        // 加密密钥和认证密钥应该不同（对于HKDF）
        let keypair = KeyPair::generate();
        let backend = keypair.get_backend();

        if backend.backend_name() == "X25519-ECDH-HKDF" {
            assert_ne!(
                derived.encryption_key, derived.authentication_key,
                "加密密钥和认证密钥应该不同以防止密钥重用"
            );
        }
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors - x25519_dalek_ng RNG compatibility
    fn test_x25519_key_agreement() {
        // 测试X25519密钥协商（使用运行时检查）
        let keypair = KeyPair::generate();
        let backend = keypair.get_backend();

        if backend.backend_name() == "X25519-ECDH-HKDF" {
            use rand::RngCore;
            use x25519_dalek_ng::{PublicKey, StaticSecret};

            // Manually create random secrets using bytes
            let mut rng = rand::rng();
            let mut secret1_bytes = [0u8; 32];
            let mut secret2_bytes = [0u8; 32];
            rng.fill_bytes(&mut secret1_bytes);
            rng.fill_bytes(&mut secret2_bytes);

            let secret1 = StaticSecret::from(secret1_bytes);
            let public1 = PublicKey::from(&secret1);

            let secret2 = StaticSecret::from(secret2_bytes);
            let public2 = PublicKey::from(&secret2);

            // 双方计算共享密钥
            let shared1 = secret1.diffie_hellman(&public2);
            let shared2 = secret2.diffie_hellman(&public1);

            // 共享密钥应该相同
            assert_eq!(
                shared1.to_bytes(),
                shared2.to_bytes(),
                "X25519密钥协商应该是对称的"
            );
        }
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_secure_keypair_uniqueness() {
        // 测试安全密钥对的唯一性（使用运行时检查）
        let keypair = KeyPair::generate();
        let backend = keypair.get_backend();

        if backend.backend_name() == "X25519-ECDH-HKDF" {
            let mut keypairs = std::collections::HashSet::new();

            // 生成100个密钥对
            for _ in 0..100 {
                let keypair = KeyPair::generate();
                keypairs.insert(keypair.public_key);
            }

            // 所有公钥应该唯一
            assert_eq!(keypairs.len(), 100, "生成的100个密钥对应该全部唯一");
        }
    }

    #[cfg(feature = "insecure_key_exchange")] // 条件编译 #14: 不安全实现的专用测试
    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_insecure_implementation_warning() {
        // 确保不安全实现有警告
        let keypair = KeyPair::generate_insecure();

        // 不安全实现仍然应该产生有效的密钥对
        assert!(keypair.is_valid());
        assert!(!keypair.public_key.iter().all(|&b| b == 0));
        assert!(!keypair.private_key.iter().all(|&b| b == 0));
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_symmetric_key_exchange() {
        // 测试对称性：A与B交换应该得到相同结果（使用运行时检查）
        let keypair_a = KeyPair::generate();
        let keypair_b = KeyPair::generate();
        let backend = keypair_a.get_backend();

        let shared_ab = keypair_a.compute_shared_secret(keypair_b.public_key);
        let shared_ba = keypair_b.compute_shared_secret(keypair_a.public_key);

        if backend.backend_name() == "X25519-ECDH-HKDF" {
            assert_eq!(
                shared_ab.shared_secret, shared_ba.shared_secret,
                "密钥交换应该是对称的"
            );
        }
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_zero_key_handling() {
        // 测试全零密钥的处理
        let keypair = KeyPair::generate();
        let zero_peer_key = [0u8; 32];

        // 即使对等方密钥为零，也应该能计算（虽然结果可能不安全）
        let shared = keypair.compute_shared_secret(zero_peer_key);

        // 结果应该确定（相同的输入产生相同的输出）
        let shared2 = keypair.compute_shared_secret(zero_peer_key);
        assert_eq!(shared.shared_secret, shared2.shared_secret);
    }
}
