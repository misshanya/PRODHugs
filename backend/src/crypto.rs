//! Password hashing — Argon2id, PHC-encoded, matching the Go service's output.
//!
//! The Go implementation produced `$argon2id$v=19$m=65536,t=3,p=2$<salt>$<hash>`
//! using `RawStdEncoding` base64. `argon2`'s `PasswordHash` round-trips that
//! exact format, so existing hashes remain verifiable after the port.

use anyhow::{anyhow, Result};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};

const MEMORY_KIB: u32 = 64 * 1024;
const ITERATIONS: u32 = 3;
const PARALLELISM: u32 = 2;
const KEY_LEN: usize = 32;

fn hasher() -> Result<Argon2<'static>> {
    let params = Params::new(MEMORY_KIB, ITERATIONS, PARALLELISM, Some(KEY_LEN))
        .map_err(|e| anyhow!("invalid argon2 params: {e}"))?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

/// Compute a PHC-encoded Argon2id hash for `password`.
pub fn generate_hash(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = hasher()?
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("hash_password: {e}"))?
        .to_string();
    Ok(hash)
}

/// Constant-time compare of `password` against a previously generated PHC hash.
pub fn compare_password_and_hash(password: &str, encoded: &str) -> Result<bool> {
    let parsed = PasswordHash::new(encoded).map_err(|e| anyhow!("parse hash: {e}"))?;
    Ok(hasher()?
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}
