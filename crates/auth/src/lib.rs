//! # ironroot-auth
//!
//! Authentication primitives for the IronRoot framework.
//!
//! ## What it provides
//!
//! - **User entities** ([`User`], [`NewUser`], [`Credentials`]).
//! - **Password hashing** ([`PasswordHasher`]) — Argon2id with sensible
//!   defaults. Argon2id is a memory-hard symmetric KDF and is **already
//!   post-quantum-secure**: Shor's algorithm does not apply, and Grover's
//!   gives at most a square-root speedup which Argon2's tuning parameters
//!   account for.
//! - **Optional post-quantum sealing** (`pq-seal` feature) — wraps the
//!   stored Argon2id hash in an authenticated envelope using **ML-KEM-768**
//!   (NIST FIPS 203) for key encapsulation and **ChaCha20-Poly1305** for
//!   authenticated encryption. This gives an additional layer of named PQ
//!   cryptography at rest: an attacker that exfiltrates only the database
//!   cannot brute-force hashes without also stealing the server's ML-KEM
//!   decapsulation key.
//! - **DAL integration** (`dal` feature) — a [`SqlUserRepository`] backed by
//!   [`ironroot_dal::Pool`] that works against SQLite, MySQL, and (with the
//!   relevant DAL feature) PostgreSQL.
//!
//! ## Quick start
//!
//! ```no_run
//! # async fn run() -> Result<(), ironroot_auth::AuthError> {
//! use ironroot_auth::{PasswordHasher, Credentials, NewUser};
//!
//! let hasher = PasswordHasher::default();
//! let new_user = NewUser {
//!     username: "ada".into(),
//!     email: "ada@example.org".into(),
//!     password: "correct horse battery staple".into(),
//! };
//! let phc = hasher.hash(&new_user.password)?;
//! assert!(hasher.verify(&new_user.password, &phc).is_ok());
//! # Ok(()) }
//! ```

mod error;
mod password;
mod user;

#[cfg(feature = "pq-seal")]
mod pq;

#[cfg(feature = "dal")]
mod repository;

#[cfg(feature = "dal")]
mod service;

pub use error::AuthError;
pub use password::PasswordHasher;
pub use user::{Credentials, NewUser, User};

#[cfg(feature = "pq-seal")]
pub use pq::{PqSealer, SealedHash};

#[cfg(feature = "dal")]
pub use repository::{SqlUserRepository, UserRepository};

#[cfg(feature = "dal")]
pub use service::AuthService;
