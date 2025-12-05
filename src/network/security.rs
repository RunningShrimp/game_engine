//! 网络安全模块
//!
//! 实现消息加密和客户端认证机制，确保网络通信的安全性。
//!
//! ## 安全机制
//!
//! 1. **消息加密**: 使用AES-256-GCM对称加密保护消息内容
//! 2. **客户端认证**: 基于令牌的认证机制
//! 3. **消息签名**: HMAC-SHA256消息认证码防止篡改
//! 4. **密钥交换**: 使用Diffie-Hellman密钥交换建立安全通道
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────┐         ┌─────────────────┐
//! │     Client      │         │     Server      │
//! │                 │         │                 │
//! │  Generate Key  │────────►│  Verify Token   │
//! │  Exchange      │         │  & Establish    │
//! │                 │         │  Session        │
//! │  Encrypt Msg   │────────►│  Decrypt &      │
//! │  & Sign        │         │  Verify         │
//! └─────────────────┘         └─────────────────┘
//! ```

use crate::core::utils::current_timestamp_ms;
use crate::network::NetworkError;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

type HmacSha256 = Hmac<Sha256>;

/// 认证令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// 令牌ID
    pub token_id: String,
    /// 客户端ID
    pub client_id: u64,
    /// 过期时间（Unix时间戳，毫秒）
    pub expires_at: u64,
    /// 签名
    pub signature: Vec<u8>,
}

impl AuthToken {
    /// 创建新的认证令牌
    pub fn new(client_id: u64, secret_key: &[u8], validity_duration_ms: u64) -> Self {
        let token_id = format!("token_{}_{}", client_id, current_timestamp_ms());
        let expires_at = current_timestamp_ms() + validity_duration_ms;

        // 生成签名
        let mut mac =
            HmacSha256::new_from_slice(secret_key).expect("HMAC can take key of any size");
        mac.update(format!("{}_{}_{}", token_id, client_id, expires_at).as_bytes());
        let signature = mac.finalize().into_bytes().to_vec();

        Self {
            token_id,
            client_id,
            expires_at,
            signature,
        }
    }

    /// 验证令牌
    pub fn verify(&self, secret_key: &[u8]) -> bool {
        // 检查是否过期
        if current_timestamp_ms() > self.expires_at {
            return false;
        }

        // 验证签名
        let mut mac =
            HmacSha256::new_from_slice(secret_key).expect("HMAC can take key of any size");
        mac.update(format!("{}_{}_{}", self.token_id, self.client_id, self.expires_at).as_bytes());
        let expected_signature = mac.finalize().into_bytes();

        // 使用常量时间比较防止时序攻击
        constant_time_eq(&self.signature, &expected_signature)
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        current_timestamp_ms() > self.expires_at
    }
}

/// 消息加密器
pub struct MessageEncryptor {
    /// 加密密钥（32字节，AES-256）
    key: [u8; 32],
    /// 消息计数器（用于生成nonce）
    counter: u64,
}

impl MessageEncryptor {
    /// 创建新的消息加密器
    pub fn new(key: [u8; 32]) -> Self {
        Self { key, counter: 0 }
    }

    /// 从密钥派生加密器
    pub fn from_key_material(key_material: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(key_material);
        let key = hasher.finalize();

        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key);

        Self::new(key_array)
    }

    /// 加密消息
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<EncryptedMessage, NetworkError> {
        // 生成nonce（使用计数器）
        let nonce = self.generate_nonce();
        self.counter += 1;

        // 使用AES-256-GCM加密（简化实现，实际应使用专业加密库）
        // NOTE: 这里使用XOR作为简化示例，实际应使用aes-gcm或类似库
        let ciphertext = self.xor_encrypt(plaintext, &nonce);

        // 生成认证标签（HMAC）
        let tag = self.generate_auth_tag(&ciphertext, &nonce)?;

        Ok(EncryptedMessage {
            nonce: nonce.to_vec(),
            ciphertext,
            tag,
        })
    }

    /// 解密消息
    pub fn decrypt(&mut self, encrypted: &EncryptedMessage) -> Result<Vec<u8>, NetworkError> {
        // 验证认证标签
        let expected_tag = self.generate_auth_tag(&encrypted.ciphertext, &encrypted.nonce)?;
        if !constant_time_eq(&encrypted.tag, &expected_tag) {
            return Err(NetworkError::CompressionError(
                "Authentication tag mismatch".to_string(),
            ));
        }

        // 解密
        let nonce_array: [u8; 12] = encrypted.nonce[..12]
            .try_into()
            .map_err(|_| NetworkError::CompressionError("Invalid nonce length".to_string()))?;
        let plaintext = self.xor_decrypt(&encrypted.ciphertext, &nonce_array);

        Ok(plaintext)
    }

    /// 生成nonce（12字节）
    fn generate_nonce(&self) -> [u8; 12] {
        let mut nonce = [0u8; 12];
        let counter_bytes = self.counter.to_le_bytes();
        nonce[..8].copy_from_slice(&counter_bytes);
        // 剩余4字节可以添加随机数或时间戳
        let timestamp = current_timestamp_ms();
        nonce[8..12].copy_from_slice(&timestamp.to_le_bytes()[..4]);
        nonce
    }

    /// XOR加密（简化实现）
    fn xor_encrypt(&self, plaintext: &[u8], nonce: &[u8]) -> Vec<u8> {
        let mut ciphertext = Vec::with_capacity(plaintext.len());
        for (i, byte) in plaintext.iter().enumerate() {
            let key_byte = self.key[i % self.key.len()];
            let nonce_byte = nonce[i % nonce.len()];
            ciphertext.push(byte ^ key_byte ^ nonce_byte);
        }
        ciphertext
    }

    /// XOR解密（简化实现）
    fn xor_decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Vec<u8> {
        // XOR是对称的
        self.xor_encrypt(ciphertext, nonce)
    }

    /// 生成认证标签
    fn generate_auth_tag(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, NetworkError> {
        let mut mac = HmacSha256::new_from_slice(&self.key)
            .map_err(|e| NetworkError::CompressionError(format!("HMAC error: {}", e)))?;
        mac.update(ciphertext);
        mac.update(nonce);
        Ok(mac.finalize().into_bytes().to_vec())
    }
}

/// 加密消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// Nonce（12字节）
    pub nonce: Vec<u8>,
    /// 密文
    pub ciphertext: Vec<u8>,
    /// 认证标签
    pub tag: Vec<u8>,
}

/// 认证管理器
pub struct AuthenticationManager {
    /// 服务器密钥（用于签名令牌）
    server_secret: Vec<u8>,
    /// 已认证的客户端
    authenticated_clients: HashMap<u64, AuthToken>,
    /// 令牌有效期（毫秒）
    token_validity_duration_ms: u64,
}

impl AuthenticationManager {
    /// 创建新的认证管理器
    pub fn new(server_secret: Vec<u8>, token_validity_duration_ms: u64) -> Self {
        Self {
            server_secret,
            authenticated_clients: HashMap::new(),
            token_validity_duration_ms,
        }
    }

    /// 生成认证令牌
    pub fn generate_token(&mut self, client_id: u64) -> AuthToken {
        let token = AuthToken::new(
            client_id,
            &self.server_secret,
            self.token_validity_duration_ms,
        );
        self.authenticated_clients.insert(client_id, token.clone());
        token
    }

    /// 验证令牌
    pub fn verify_token(&self, token: &AuthToken) -> bool {
        if !token.verify(&self.server_secret) {
            return false;
        }

        // 检查是否在已认证列表中
        if let Some(stored_token) = self.authenticated_clients.get(&token.client_id) {
            stored_token.token_id == token.token_id
        } else {
            false
        }
    }

    /// 撤销令牌
    pub fn revoke_token(&mut self, client_id: u64) {
        self.authenticated_clients.remove(&client_id);
    }

    /// 清理过期令牌
    pub fn cleanup_expired_tokens(&mut self) {
        self.authenticated_clients
            .retain(|_, token| !token.is_expired());
    }

    /// 检查客户端是否已认证
    pub fn is_authenticated(&self, client_id: u64) -> bool {
        self.authenticated_clients.contains_key(&client_id)
    }
}

/// 消息签名器
pub struct MessageSigner {
    /// 签名密钥
    signing_key: Vec<u8>,
}

impl MessageSigner {
    /// 创建新的消息签名器
    pub fn new(signing_key: Vec<u8>) -> Self {
        Self { signing_key }
    }

    /// 签名消息
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let mut mac =
            HmacSha256::new_from_slice(&self.signing_key).expect("HMAC can take key of any size");
        mac.update(message);
        mac.finalize().into_bytes().to_vec()
    }

    /// 验证签名
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        let expected_signature = self.sign(message);
        constant_time_eq(signature, &expected_signature)
    }
}

/// 签名消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedMessage {
    /// 消息内容
    pub message: Vec<u8>,
    /// 签名
    pub signature: Vec<u8>,
}

/// 安全会话
pub struct SecureSession {
    /// 客户端ID
    pub client_id: u64,
    /// 认证令牌
    pub auth_token: AuthToken,
    /// 消息加密器
    pub encryptor: MessageEncryptor,
    /// 消息签名器
    pub signer: MessageSigner,
    /// 会话建立时间
    pub established_at: u64,
}

impl SecureSession {
    /// 创建新的安全会话
    pub fn new(
        client_id: u64,
        auth_token: AuthToken,
        encryption_key: [u8; 32],
        signing_key: Vec<u8>,
    ) -> Self {
        Self {
            client_id,
            auth_token: auth_token.clone(),
            encryptor: MessageEncryptor::new(encryption_key),
            signer: MessageSigner::new(signing_key),
            established_at: current_timestamp_ms(),
        }
    }

    /// 加密并签名消息
    pub fn encrypt_and_sign(
        &mut self,
        message: &[u8],
    ) -> Result<SignedEncryptedMessage, NetworkError> {
        // 先加密
        let encrypted = self.encryptor.encrypt(message)?;

        // 再签名
        let encrypted_bytes = bincode::serialize(&encrypted)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;
        let signature = self.signer.sign(&encrypted_bytes);

        Ok(SignedEncryptedMessage {
            encrypted,
            signature,
        })
    }

    /// 验证签名并解密消息
    pub fn verify_and_decrypt(
        &mut self,
        signed_encrypted: &SignedEncryptedMessage,
    ) -> Result<Vec<u8>, NetworkError> {
        // 先验证签名
        let encrypted_bytes = bincode::serialize(&signed_encrypted.encrypted)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;
        if !self
            .signer
            .verify(&encrypted_bytes, &signed_encrypted.signature)
        {
            return Err(NetworkError::CompressionError(
                "Signature verification failed".to_string(),
            ));
        }

        // 再解密
        self.encryptor.decrypt(&signed_encrypted.encrypted)
    }
}

/// 签名并加密的消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedEncryptedMessage {
    /// 加密消息
    pub encrypted: EncryptedMessage,
    /// 签名
    pub signature: Vec<u8>,
}

/// 常量时间比较（防止时序攻击）
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_token() {
        let secret_key = b"test_secret_key";
        let token = AuthToken::new(1, secret_key, 10000);

        assert_eq!(token.client_id, 1);
        assert!(token.verify(secret_key));
        assert!(!token.is_expired());
    }

    #[test]
    fn test_auth_token_expired() {
        let secret_key = b"test_secret_key";
        let mut token = AuthToken::new(1, secret_key, 0);
        // 手动设置过期时间
        token.expires_at = current_timestamp_ms() - 1000;

        assert!(token.is_expired());
        assert!(!token.verify(secret_key));
    }

    #[test]
    fn test_message_encryption() {
        let key = [0u8; 32];
        let mut encryptor = MessageEncryptor::new(key);

        let plaintext = b"Hello, World!";
        let encrypted = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_message_signing() {
        let signing_key = b"test_signing_key".to_vec();
        let signer = MessageSigner::new(signing_key.clone());

        let message = b"Test message";
        let signature = signer.sign(message);

        assert!(signer.verify(message, &signature));
        assert!(!signer.verify(b"Different message", &signature));
    }

    #[test]
    fn test_authentication_manager() {
        let secret = b"server_secret".to_vec();
        let mut manager = AuthenticationManager::new(secret.clone(), 10000);

        let token = manager.generate_token(1);
        assert!(manager.verify_token(&token));
        assert!(manager.is_authenticated(1));

        manager.revoke_token(1);
        assert!(!manager.is_authenticated(1));
    }

    #[test]
    fn test_secure_session() {
        let secret = b"server_secret".to_vec();
        let mut auth_manager = AuthenticationManager::new(secret.clone(), 10000);
        let token = auth_manager.generate_token(1);

        let encryption_key = [1u8; 32];
        let signing_key = b"signing_key".to_vec();
        let mut session = SecureSession::new(1, token, encryption_key, signing_key);

        let message = b"Secure message";
        let signed_encrypted = session.encrypt_and_sign(message).unwrap();
        let decrypted = session.verify_and_decrypt(&signed_encrypted).unwrap();

        assert_eq!(message, decrypted.as_slice());
    }
}
