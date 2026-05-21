//! High-level [`AuthService`] composing the password hasher, optional PQ
//! sealer, and the user repository.

use crate::{AuthError, Credentials, NewUser, PasswordHasher, User, UserRepository};

#[cfg(feature = "pq-seal")]
use std::sync::Arc;
#[cfg(feature = "pq-seal")]
use crate::{pq::SealedHash, PqSealer};

/// Orchestrates user registration and credential verification.
///
/// The service is generic over any [`UserRepository`] so it can be tested
/// in-process with an in-memory implementation or wired to
/// [`crate::SqlUserRepository`] in production.
pub struct AuthService<R: UserRepository> {
    repo: R,
    hasher: PasswordHasher,
    #[cfg(feature = "pq-seal")]
    sealer: Option<Arc<PqSealer>>,
}

impl<R: UserRepository> AuthService<R> {
    /// Build a service from a repository, using default Argon2id parameters
    /// and no PQ sealing.
    pub fn new(repo: R) -> Self {
        Self {
            repo,
            hasher: PasswordHasher::default(),
            #[cfg(feature = "pq-seal")]
            sealer: None,
        }
    }

    /// Override the password hasher (custom Argon2id parameters).
    pub fn with_hasher(mut self, hasher: PasswordHasher) -> Self {
        self.hasher = hasher;
        self
    }

    /// Enable post-quantum sealing of stored password hashes.
    #[cfg(feature = "pq-seal")]
    pub fn with_pq_sealer(mut self, sealer: Arc<PqSealer>) -> Self {
        self.sealer = Some(sealer);
        self
    }

    /// Register a new user. Validates the input, hashes the password with
    /// Argon2id, optionally PQ-seals the hash, and persists the user.
    pub async fn register(&self, new_user: NewUser) -> Result<User, AuthError> {
        new_user.validate()?;

        if self
            .repo
            .find_by_username(&new_user.username)
            .await?
            .is_some()
        {
            return Err(AuthError::AlreadyExists);
        }

        let phc = self.hasher.hash(&new_user.password)?;
        let stored = self.store_form(&phc)?;
        tracing::debug!(username = %new_user.username, "ironroot-auth: registering user");

        self.repo
            .create(&new_user.username, &new_user.email, &stored)
            .await
    }

    /// Verify credentials. Returns the [`User`] on success, or
    /// [`AuthError::InvalidCredentials`] on any mismatch.
    pub async fn verify(&self, creds: &Credentials) -> Result<User, AuthError> {
        let user = self
            .repo
            .find_by_username(&creds.username)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        let phc = self.load_form(&user.password_hash)?;
        self.hasher.verify(&creds.password, &phc)?;
        Ok(user)
    }

    fn store_form(&self, phc: &str) -> Result<String, AuthError> {
        #[cfg(feature = "pq-seal")]
        {
            if let Some(sealer) = self.sealer.as_ref() {
                let sealed = sealer.seal(phc.as_bytes())?;
                return Ok(sealed.encode());
            }
        }
        Ok(phc.to_string())
    }

    fn load_form(&self, stored: &str) -> Result<String, AuthError> {
        #[cfg(feature = "pq-seal")]
        {
            if let Some(sealer) = self.sealer.as_ref() {
                let sealed = SealedHash::decode(stored)?;
                let bytes = sealer.unseal(&sealed)?;
                return String::from_utf8(bytes)
                    .map_err(|e| AuthError::PqSeal(format!("non-utf8 payload: {e}")));
            }
        }
        Ok(stored.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemRepo {
        users: Mutex<Vec<User>>,
        next_id: Mutex<i64>,
    }

    #[async_trait]
    impl UserRepository for MemRepo {
        async fn create(
            &self,
            username: &str,
            email: &str,
            password_hash: &str,
        ) -> Result<User, AuthError> {
            let mut users = self.users.lock().unwrap();
            if users.iter().any(|u| u.username == username) {
                return Err(AuthError::AlreadyExists);
            }
            let mut id_guard = self.next_id.lock().unwrap();
            *id_guard += 1;
            let user = User {
                id: *id_guard,
                username: username.to_string(),
                email: email.to_string(),
                password_hash: password_hash.to_string(),
                created_at: 0,
            };
            users.push(user.clone());
            Ok(user)
        }

        async fn find_by_username(&self, username: &str) -> Result<Option<User>, AuthError> {
            Ok(self
                .users
                .lock()
                .unwrap()
                .iter()
                .find(|u| u.username == username)
                .cloned())
        }

        async fn find_by_id(&self, id: i64) -> Result<Option<User>, AuthError> {
            Ok(self
                .users
                .lock()
                .unwrap()
                .iter()
                .find(|u| u.id == id)
                .cloned())
        }

        async fn delete(&self, id: i64) -> Result<(), AuthError> {
            self.users.lock().unwrap().retain(|u| u.id != id);
            Ok(())
        }
    }

    #[tokio::test]
    async fn register_then_verify() {
        let svc = AuthService::new(MemRepo::default());
        let u = svc
            .register(NewUser {
                username: "ada".into(),
                email: "ada@example.org".into(),
                password: "correcthorse".into(),
            })
            .await
            .unwrap();
        assert_eq!(u.username, "ada");

        let v = svc
            .verify(&Credentials {
                username: "ada".into(),
                password: "correcthorse".into(),
            })
            .await
            .unwrap();
        assert_eq!(v.id, u.id);

        let bad = svc
            .verify(&Credentials {
                username: "ada".into(),
                password: "wrongguess".into(),
            })
            .await;
        assert!(matches!(bad, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn duplicate_registration_rejected() {
        let svc = AuthService::new(MemRepo::default());
        svc.register(NewUser {
            username: "ada".into(),
            email: "a@b.c".into(),
            password: "longenough".into(),
        })
        .await
        .unwrap();
        let dup = svc
            .register(NewUser {
                username: "ada".into(),
                email: "a@b.c".into(),
                password: "longenough".into(),
            })
            .await;
        assert!(matches!(dup, Err(AuthError::AlreadyExists)));
    }

    #[cfg(feature = "pq-seal")]
    #[tokio::test]
    async fn pq_sealed_register_and_verify() {
        let sealer = Arc::new(PqSealer::generate());
        let svc = AuthService::new(MemRepo::default()).with_pq_sealer(sealer);
        let u = svc
            .register(NewUser {
                username: "ada".into(),
                email: "ada@example.org".into(),
                password: "correcthorse".into(),
            })
            .await
            .unwrap();
        // The stored form must NOT look like a plain Argon2id PHC string.
        assert!(!u.password_hash.starts_with("$argon2id$"));

        let v = svc
            .verify(&Credentials {
                username: "ada".into(),
                password: "correcthorse".into(),
            })
            .await
            .unwrap();
        assert_eq!(v.username, "ada");
    }
}
