//! Error type for the auth crate.

use thiserror::Error;

/// Top-level error type for `ironroot-auth`.
#[derive(Debug, Error)]
pub enum AuthError {
    /// The provided credentials did not match a stored user.
    #[error("invalid credentials")]
    InvalidCredentials,

    /// A user with the same primary key or unique field already exists.
    #[error("user already exists")]
    AlreadyExists,

    /// The requested user could not be found.
    #[error("user not found")]
    NotFound,

    /// Username/email/password failed validation.
    #[error("validation error: {0}")]
    Validation(String),

    /// Failure while hashing or verifying a password.
    #[error("password hashing error: {0}")]
    Hash(String),

    /// Failure inside the post-quantum sealing layer.
    #[cfg(feature = "pq-seal")]
    #[error("post-quantum sealing error: {0}")]
    PqSeal(String),

    /// Failure in the underlying data access layer.
    #[cfg(feature = "dal")]
    #[error("dal error: {0}")]
    Dal(#[from] ironroot_dal::DalError),
}
