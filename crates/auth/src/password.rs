//! Argon2id-based password hashing.

use argon2::{Algorithm, Argon2, Params, Version};
use password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString};
use rand_core::OsRng;

use crate::AuthError;

/// Password hasher using Argon2id.
///
/// Argon2id is a memory-hard symmetric KDF and is **already
/// post-quantum-secure** in the threat model that matters for password
/// hashing: no public-key cryptography is involved (so Shor's algorithm does
/// not apply), and the symmetric primitives only benefit from a square-root
/// speedup under Grover's algorithm — which the memory-hardness parameters
/// already absorb.
///
/// For an *additional* PQ layer (sealing the resulting hash at rest under an
/// ML-KEM-encapsulated key) see [`crate::PqSealer`].
#[derive(Debug, Clone)]
pub struct PasswordHasher {
    params: Params,
}

impl Default for PasswordHasher {
    fn default() -> Self {
        // RFC 9106 recommended defaults for interactive scenarios.
        let params = Params::new(
            19 * 1024, // m_cost (KiB) — 19 MiB
            2,         // t_cost
            1,         // p_cost
            None,      // default output length (32 bytes)
        )
        .expect("argon2 params");
        Self { params }
    }
}

impl PasswordHasher {
    /// Construct a hasher with custom Argon2id parameters.
    pub fn with_params(params: Params) -> Self {
        Self { params }
    }

    /// Hash a plain-text password into an Argon2id PHC string.
    pub fn hash(&self, password: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, self.params.clone());
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthError::Hash(e.to_string()))?;
        Ok(hash.to_string())
    }

    /// Verify a plain-text password against a stored Argon2id PHC string.
    pub fn verify(&self, password: &str, phc: &str) -> Result<(), AuthError> {
        let parsed = PasswordHash::new(phc).map_err(|e| AuthError::Hash(e.to_string()))?;
        let argon2 = Argon2::default();
        argon2
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| AuthError::InvalidCredentials)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_roundtrip() {
        let h = PasswordHasher::default();
        let phc = h.hash("correct horse battery staple").unwrap();
        assert!(phc.starts_with("$argon2id$"));
        h.verify("correct horse battery staple", &phc).unwrap();
    }

    #[test]
    fn verify_rejects_wrong_password() {
        let h = PasswordHasher::default();
        let phc = h.hash("hunter2hunter2").unwrap();
        let err = h.verify("password1234", &phc).unwrap_err();
        assert!(matches!(err, AuthError::InvalidCredentials));
    }

    #[test]
    fn distinct_salts_produce_distinct_hashes() {
        let h = PasswordHasher::default();
        let a = h.hash("samepassword").unwrap();
        let b = h.hash("samepassword").unwrap();
        assert_ne!(a, b);
    }
}
