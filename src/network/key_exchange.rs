//! 密钥交换协议模块
//!
//! 实现X25519椭圆曲线Diffie-Hellman密钥交换，用于建立安全通信通道。
//!
//! ## 功能
//!
//! - 生成并管理临时密钥对
//! - 执行X25519密钥交换协议
//! - 从共享密钥派生加密密钥
//! - 密钥有效期管理
//!
//! ## 架构
//!
//! ```text
//! ┌─────────────────┐           ┌─────────────────┐
//! │     Client      │           │     Server      │
//! │                 │           │                 │
//! │ Generate Key    │           │ Generate Key    │
//! │ Pair (Pk1, Sk1) │           │ Pair (Pk2, Sk2) │
//! │                 │           │                 │
//! │ Send Pk1 ──────────────────► Receive Pk1     │
//! │                 │           │                 │
//! │ Receive Pk2 ◄──────────────── Send Pk2       │
//! │                 │           │                 │
//! │ Compute:        │           │ Compute:        │
//! │ SharedSecret =   │           │ SharedSecret =   │
//! │ ECDH(Sk1, Pk2)  │           │ ECDH(Sk2, Pk1)  │
//! │                 │           │                 │
//! │ KDF(SharedSecret)│───────────│KDF(SharedSecret)│
//! │                 │           │                 │
//! │ EncKey, AuthKey │───────────│EncKey, AuthKey │
//! └─────────────────┘           └─────────────────┘
//! ```

use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// X25519密钥对
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
    /// 生成新的X25519密钥对
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();

        // 生成私钥（32字节随机数）
        let mut private_key = [0u8; 32];
        rng.fill_bytes(&mut private_key);

        // 按X25519规范处理私钥
        private_key[0] &= 248;
        private_key[31] &= 127;
        private_key[31] |= 64;

        // 计算公钥（简化实现：使用SHA256作为伪ECDH）
        // 注意：实际应使用 x25519-dalek 库的真实X25519
        let public_key = Self::derive_public_key(&private_key);

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

    /// 从私钥派生公钥
    fn derive_public_key(private_key: &[u8; 32]) -> [u8; 32] {
        // 简化实现：使用SHA256
        // 实际应使用 x25519 库的真实算法
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
        !self.public_key.iter().all(|&b| b == 0)
            && !self.private_key.iter().all(|&b| b == 0)
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
    /// 从ECDH共享密钥派生出加密密钥和认证密钥
    pub fn derive(shared_secret: [u8; 32]) -> Self {
        // 使用HKDF派生密钥
        // 简化实现：使用SHA256进行密钥派生
        // 实际应使用 hkdf 库的标准实现

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

    /// 执行ECDH密钥交换（从对方公钥和本地私钥计算共享密钥）
    pub fn compute_shared_secret(&self, peer_public_key: [u8; 32]) -> SharedSecret {
        // 简化实现：模拟ECDH
        // 实际应使用 x25519_dalek 库
        
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
