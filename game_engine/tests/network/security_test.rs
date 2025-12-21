//! 网络安全测试
//!
//! 测试密钥交换、消息加密和认证机制。

use game_engine::network::security::AuthToken;

#[test]
fn test_auth_token_creation() {
    let secret_key = b"test_secret_key_32_bytes_long!!";
    let client_id = 12345;
    let validity_duration_ms = 3600000; // 1小时

    let token = AuthToken::new(client_id, secret_key, validity_duration_ms);

    assert_eq!(token.client_id, client_id);
    assert_eq!(token.version, 1);
    assert!(!token.token_id.is_empty());
    assert!(token.expires_at > 0);
    assert!(!token.signature.is_empty());
}

#[test]
fn test_auth_token_verification() {
    let secret_key = b"test_secret_key_32_bytes_long!!";
    let client_id = 12345;
    let validity_duration_ms = 3600000;

    let token = AuthToken::new(client_id, secret_key, validity_duration_ms);

    // 使用正确的密钥验证
    assert!(token.verify(secret_key));

    // 使用错误的密钥验证
    let wrong_key = b"wrong_secret_key_32_bytes_long!!";
    assert!(!token.verify(wrong_key));
}

#[test]
fn test_auth_token_expiration() {
    let secret_key = b"test_secret_key_32_bytes_long!!";
    let client_id = 12345;
    let validity_duration_ms = 100; // 100毫秒

    let token = AuthToken::new(client_id, secret_key, validity_duration_ms);

    // 立即验证应该成功
    assert!(token.verify(secret_key));

    // 注意：我们无法直接测试过期，因为需要等待时间
    // 但可以验证is_expired方法存在
    let is_expired = token.is_expired();
    assert!(!is_expired); // 刚创建的token不应该过期
}

#[test]
fn test_auth_token_serialization() {
    use serde_json;

    let secret_key = b"test_secret_key_32_bytes_long!!";
    let token = AuthToken::new(12345, secret_key, 3600000);

    // 测试序列化
    let serialized = serde_json::to_string(&token);
    assert!(serialized.is_ok());

    // 测试反序列化
    if let Ok(json_str) = serialized {
        let deserialized: Result<AuthToken, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        if let Ok(deserialized_token) = deserialized {
            assert_eq!(deserialized_token.client_id, token.client_id);
            assert_eq!(deserialized_token.version, token.version);
            assert_eq!(deserialized_token.token_id, token.token_id);
        }
    }
}

#[test]
fn test_auth_token_version() {
    let secret_key = b"test_secret_key_32_bytes_long!!";
    let token = AuthToken::new(12345, secret_key, 3600000);

    // 验证版本号
    assert_eq!(token.version, 1);
}

#[test]
fn test_auth_token_unique_ids() {
    let secret_key = b"test_secret_key_32_bytes_long!!";
    
    let token1 = AuthToken::new(1, secret_key, 3600000);
    let token2 = AuthToken::new(2, secret_key, 3600000);

    // 不同客户端ID应该生成不同的token ID
    assert_ne!(token1.token_id, token2.token_id);
    assert_ne!(token1.client_id, token2.client_id);
}

#[test]
fn test_auth_token_signature_verification() {
    let secret_key = b"test_secret_key_32_bytes_long!!";
    let token = AuthToken::new(12345, secret_key, 3600000);

    // 验证签名长度（HMAC-SHA256应该是32字节）
    assert_eq!(token.signature.len(), 32);
}

