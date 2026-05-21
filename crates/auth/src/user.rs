//! User-facing entity types.

use crate::AuthError;

/// A persisted user.
///
/// `password_hash` is an Argon2id PHC string (or its sealed form when the
/// `pq-seal` feature is enabled — see [`crate::PqSealer`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    /// Primary key.
    pub id: i64,
    /// Unique, case-sensitive username.
    pub username: String,
    /// Contact email.
    pub email: String,
    /// Opaque password hash (Argon2id PHC string, possibly PQ-sealed).
    pub password_hash: String,
    /// Creation time as a Unix epoch in seconds.
    pub created_at: i64,
}

/// Input to [`crate::AuthService::register`].
#[derive(Debug, Clone)]
pub struct NewUser {
    /// Desired username.
    pub username: String,
    /// Contact email.
    pub email: String,
    /// Plain-text password — never persisted.
    pub password: String,
}

impl NewUser {
    /// Apply basic input validation.
    pub fn validate(&self) -> Result<(), AuthError> {
        validate_username(&self.username)?;
        validate_email(&self.email)?;
        validate_password(&self.password)?;
        Ok(())
    }
}

/// Credentials presented at sign-in.
#[derive(Debug, Clone)]
pub struct Credentials {
    /// Username supplied by the client.
    pub username: String,
    /// Plain-text password supplied by the client.
    pub password: String,
}

fn validate_username(s: &str) -> Result<(), AuthError> {
    if s.len() < 3 || s.len() > 64 {
        return Err(AuthError::Validation(
            "username must be 3..=64 characters".into(),
        ));
    }
    let ok = s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.');
    if !ok {
        return Err(AuthError::Validation(
            "username may only contain ASCII letters, digits, '_', '-', or '.'".into(),
        ));
    }
    Ok(())
}

fn validate_email(s: &str) -> Result<(), AuthError> {
    // Deliberately permissive — full RFC 5321 validation is the wrong layer here.
    if !s.contains('@') || s.len() < 3 || s.len() > 320 {
        return Err(AuthError::Validation(
            "email must contain '@' and be 3..=320 characters".into(),
        ));
    }
    Ok(())
}

fn validate_password(s: &str) -> Result<(), AuthError> {
    if s.len() < 8 {
        return Err(AuthError::Validation(
            "password must be at least 8 characters".into(),
        ));
    }
    if s.len() > 1024 {
        return Err(AuthError::Validation(
            "password must be at most 1024 characters".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_good_input() {
        let u = NewUser {
            username: "ada".into(),
            email: "ada@example.org".into(),
            password: "correcthorse".into(),
        };
        u.validate().unwrap();
    }

    #[test]
    fn rejects_short_username() {
        let u = NewUser {
            username: "a".into(),
            email: "a@b.c".into(),
            password: "longenough".into(),
        };
        assert!(matches!(u.validate(), Err(AuthError::Validation(_))));
    }

    #[test]
    fn rejects_bad_email() {
        let u = NewUser {
            username: "ada".into(),
            email: "not-an-email".into(),
            password: "longenough".into(),
        };
        assert!(matches!(u.validate(), Err(AuthError::Validation(_))));
    }

    #[test]
    fn rejects_short_password() {
        let u = NewUser {
            username: "ada".into(),
            email: "a@b.c".into(),
            password: "short".into(),
        };
        assert!(matches!(u.validate(), Err(AuthError::Validation(_))));
    }
}
