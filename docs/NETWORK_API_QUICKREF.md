# Network Module API Quick Reference

## New KeyPair Methods

### KeyPair::compute_shared_secret()

```rust
impl KeyPair {
    /// Calculate shared secret from peer's public key and local private key
    pub fn compute_shared_secret(&self, peer_public_key: [u8; 32]) -> SharedSecret
}
```

**Usage Example:**
```rust
use game_engine::network::key_exchange::KeyPair;

// Generate two keypairs
let alice_keypair = KeyPair::generate();
let bob_keypair = KeyPair::generate();

// Alice computes shared secret using Bob's public key
let alice_shared = alice_keypair.compute_shared_secret(bob_keypair.public_key);

// Bob computes shared secret using Alice's public key
let bob_shared = bob_keypair.compute_shared_secret(alice_keypair.public_key);

// Both should get the same shared secret (with secure_key_exchange feature)
assert_eq!(alice_shared.shared_secret, bob_shared.shared_secret);
```

**Returns:**
- `SharedSecret` containing:
  - `shared_secret: [u8; 32]` - The raw shared secret
  - `encryption_key: [u8; 32]` - Derived encryption key
  - `authentication_key: [u8; 32]` - Derived authentication key

### KeyPair::generate_insecure()

```rust
impl KeyPair {
    /// Generate insecure keypair for testing only
    #[cfg(feature = "insecure_key_exchange")]
    pub fn generate_insecure() -> Self
}
```

**Usage Example:**
```rust
#[cfg(feature = "insecure_key_exchange")]
{
    use game_engine::network::key_exchange::KeyPair;
    
    // WARNING: Only for testing!
    let keypair = KeyPair::generate_insecure();
    assert!(keypair.is_valid());
}
```

**Warning:** This method is only available with the `insecure_key_exchange` feature and should NEVER be used in production!

## Feature Flags

### secure_key_exchange (default)
Uses X25519 ECDH for key exchange and HKDF for key derivation.

```toml
[dependencies]
game_engine = { version = "0.1.0", features = ["secure_key_exchange"] }
```

### insecure_key_exchange
Uses simplified SHA256-based implementation. Only for testing!

```toml
[dependencies]
game_engine = { version = "0.1.0", features = ["insecure_key_exchange"] }
```

## Complete KeyPair API

```rust
pub struct KeyPair {
    pub public_key: [u8; 32],
    pub private_key: [u8; 32],
    pub created_at: u64,
}

impl KeyPair {
    // Generate new keypair
    pub fn generate() -> Self;
    
    // Generate insecure keypair (testing only)
    #[cfg(feature = "insecure_key_exchange")]
    pub fn generate_insecure() -> Self;
    
    // Check if keypair is valid
    pub fn is_valid(&self) -> bool;
    
    // Get keypair age in seconds
    pub fn age_secs(&self) -> u64;
    
    // Compute shared secret from peer's public key
    pub fn compute_shared_secret(&self, peer_public_key: [u8; 32]) -> SharedSecret;
}
```

## Related Types

### SharedSecret
```rust
pub struct SharedSecret {
    pub shared_secret: [u8; 32],        // Raw shared secret
    pub encryption_key: [u8; 32],       // Derived encryption key
    pub authentication_key: [u8; 32],   // Derived authentication key
}

impl SharedSecret {
    pub fn derive(shared_secret: [u8; 32]) -> Self;
}
```

### KeyExchange
```rust
pub struct KeyExchange {
    local_keypair: KeyPair,
}

impl KeyExchange {
    pub fn new() -> Self;
    pub fn public_key(&self) -> [u8; 32];
    pub fn compute_shared_secret(&self, peer_public_key: [u8; 32]) -> SharedSecret;
    pub fn keypair(&self) -> &KeyPair;
}
```

## Testing Tips

1. **Use secure_key_exchange for production tests**
   ```rust
   let keypair = KeyPair::generate();
   let shared = keypair.compute_shared_secret(peer_key);
   // Both parties get the same result
   assert_eq!(shared.shared_secret, expected);
   ```

2. **Test key agreement symmetry**
   ```rust
   let keypair_a = KeyPair::generate();
   let keypair_b = KeyPair::generate();
   
   let shared_ab = keypair_a.compute_shared_secret(keypair_b.public_key);
   let shared_ba = keypair_b.compute_shared_secret(keypair_a.public_key);
   
   assert_eq!(shared_ab.shared_secret, shared_ba.shared_secret);
   ```

3. **Test key derivation**
   ```rust
   let shared = keypair.compute_shared_secret(peer_key);
   // Encryption and authentication keys should be different
   assert_ne!(shared.encryption_key, shared.authentication_key);
   ```

4. **Test keypair serialization**
   ```rust
   let keypair1 = KeyPair::generate();
   let serialized = bincode::serialize(&keypair1)?;
   let keypair2: KeyPair = bincode::deserialize(&serialized)?;
   assert_eq!(keypair1.public_key, keypair2.public_key);
   ```

## Migration Guide

If you were using `KeyExchange` and want to switch to `KeyPair`:

**Before:**
```rust
let ke = KeyExchange::new();
let shared = ke.compute_shared_secret(peer_public_key);
let keypair = ke.keypair().unwrap();
```

**After:**
```rust
let keypair = KeyPair::generate();
let shared = keypair.compute_shared_secret(peer_public_key);
```

The API is now more direct and doesn't require the intermediate `KeyExchange` wrapper!
