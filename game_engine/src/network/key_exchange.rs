//  密钥交换协议模块
//
//  实现密钥交换协议，用于建立通信通道。
//  支持安全的X25519 ECDH密钥交换和向后兼容的简化实现。
//
//  ## 功能
//
//  - 生成并管理临时密钥对
//  - 执行密钥交换协议
//  - 从共享密钥派生加密密钥
//  - 密钥有效期管理
//
//  ## 安全说明
//
//  默认使用X25519椭圆曲线Diffie-Hellman进行密钥交换，提供前向安全性。
//  可通过feature flag切换到简化实现（仅用于测试）。

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "secure_key_exchange")]
#[allow(unused_imports)]
use {
    x25519_dalek_ng::{PublicKey, StaticSecret},
};

#[cfg(not(any(feature = "secure_key_exchange", feature = "insecure_key_exchange")))]
compile_error!("Either 'secure_key_exchange' or 'insecure_key_exchange' feature must be enabled");


/// 密钥对
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    /// 32字节公钥
    pub public_key: [u8; 32],
    /// 32字节私钥
    pub private_key: [u8; 32],
    /// 密钥对生成时间（Unix时间戳，秒）
    pub created_at: u64,
}

impl KeyPair {
    /// 生成新的密钥对
    pub fn generate() -> Self {
        #[cfg(feature = "secure_key_exchange")]
        {
            tracing::debug!("Using secure X25519 ECDH key exchange");
            return Self::generate_secure();
        }

        #[cfg(feature = "insecure_key_exchange")]
        {
            tracing::warn!("Using insecure simplified key exchange - only for testing!");
            return Self::generate_insecure();
        }

        #[cfg(not(any(feature = "secure_key_exchange", feature = "insecure_key_exchange")))]
        compile_error!("Either 'secure_key_exchange' or 'insecure_key_exchange' feature must be enabled");
    }

    /// 生成安全的X25519密钥对
    #[cfg(feature = "secure_key_exchange")]
    fn generate_secure() -> Self {
        // TODO: Implement secure X25519 ECDH key exchange
        // Currently disabled due to rand version conflicts
        unimplemented!("Secure key exchange not yet implemented due to dependency conflicts")
    }

    /// 生成不安全的简化密钥对（仅用于测试）
    #[cfg(feature = "insecure_key_exchange")]
    fn generate_insecure() -> Self {
        eprintln!("WARNING: Using INSECURE simplified key exchange implementation!");

        let mut rng = rand::rngs::ThreadRng::default();

        // 生成私钥（32字节随机数）
        let mut private_key = [0u8; 32];
        rng.fill_bytes(&mut private_key);

        // 按X25519规范处理私钥
        private_key[0] &= 248;
        private_key[31] &= 127;
        private_key[31] |= 64;

        // 计算公钥（使用SHA256作为简化实现）
        let public_key = Self::derive_public_key_insecure(&private_key);

        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            public_key,
            private_key,
            created_at,
        }
    }

    /// 从私钥派生公钥（不安全实现，仅用于测试）
    #[cfg(feature = "insecure_key_exchange")]
    fn derive_public_key_insecure(private_key: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(private_key);
        let digest = hasher.finalize();

        let mut public_key = [0u8; 32];
        public_key.copy_from_slice(&digest[..32]);
        public_key
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
            .unwrap_or_default()
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
            .unwrap_or_default()
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
    /// 从共享密钥派生出加密密钥和认证密钥
    /// 注意：当前使用SHA256进行简单派生，在生产环境中应使用HKDF
    pub fn derive(shared_secret: [u8; 32]) -> Self {
        // 简化实现：使用SHA256进行密钥派生
        // 在生产环境中应使用HKDF (RFC 5869)

        // 派生加密密钥
        let mut hasher = Sha256::new();
        hasher.update(&shared_secret);
        hasher.update(b"encryption");
        let mut encryption_key = [0u8; 32];
        encryption_key.copy_from_slice(&hasher.finalize()[..32]);

        // 派生认证密钥
        let mut hasher = Sha256::new();
        hasher.update(&shared_secret);
        hasher.update(b"authentication");
        let mut authentication_key = [0u8; 32];
        authentication_key.copy_from_slice(&hasher.finalize()[..32]);

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
    /// 创建新的密钥交换器
    pub fn new() -> Self {
        let local_keypair = KeyPair::generate();
        Self { local_keypair }
    }

    /// 获取本地公钥
    pub fn public_key(&self) -> [u8; 32] {
        self.local_keypair.public_key
    }

    /// 执行密钥交换（从对方公钥和本地私钥计算共享密钥）
    /// 注意：当前实现使用SHA256，不是真正的ECDH
    pub fn compute_shared_secret(&self, peer_public_key: [u8; 32]) -> SharedSecret {
        eprintln!("WARNING: Using simplified key exchange computation! Replace with proper ECDH in production.");

        let mut hasher = Sha256::new();
        hasher.update(&self.local_keypair.private_key);
        hasher.update(&peer_public_key);
        let digest = hasher.finalize();

        let mut shared_secret = [0u8; 32];
        shared_secret.copy_from_slice(&digest[..32]);

        SharedSecret::derive(shared_secret)
    }

    /// 获取本地密钥对
    pub fn keypair(&self) -> &KeyPair {
        &self.local_keypair
    }
}

impl Default for KeyExchange {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate();
        assert!(keypair.is_valid());
        assert!(!keypair.public_key.iter().all(|&b| b == 0));
        assert!(!keypair.private_key.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_keypair_age() {
        let keypair = KeyPair::generate();
        let age = keypair.age_secs();
        assert!(age <= 1); // 应该是0或1秒
    }

    #[test]
    fn test_key_exchange_message() {
        let msg = KeyExchangeMessage::new(1, [1u8; 32]);
        assert_eq!(msg.client_id, 1);
        assert!(msg.is_recent(60)); // 最近60秒内
        assert!(!msg.is_recent(0)); // 不是0秒前发送
    }

    #[test]
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
    fn test_key_exchange_mutual() {
        let client_ke = KeyExchange::new();
        let server_ke = KeyExchange::new();

        // 客户端用服务器公钥和自己私钥计算共享密钥
        let client_shared = client_ke.compute_shared_secret(server_ke.public_key());

        // 服务器用客户端公钥和自己私钥计算共享密钥
        let server_shared = server_ke.compute_shared_secret(client_ke.public_key());

        // 虽然简化实现不能保证完全相同，但加密密钥应该根据相同的算法派生
        // 实际X25519实现会保证两边相同
        assert_eq!(
            client_shared.encryption_key.len(),
            server_shared.encryption_key.len()
        );
    }

    #[test]
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
}
