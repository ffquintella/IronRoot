# `ironroot-auth`

Authentication primitives for IronRoot — user entities, password hashing,
and an optional post-quantum sealing layer.

## What's in it

- **`User`, `NewUser`, `Credentials`** — domain types with validation.
- **`PasswordHasher`** — Argon2id with RFC 9106 interactive defaults.
- **`PqSealer`** *(feature `pq-seal`)* — wraps stored hashes in an
  ML-KEM-768 + ChaCha20-Poly1305 envelope for at-rest PQ protection.
- **`UserRepository`** trait + **`SqlUserRepository`** *(feature `dal`)* —
  persistence backed by [`ironroot-dal`](dal.md).
- **`AuthService`** *(feature `dal`)* — composes hasher, repo, and optional
  sealer; the entry point for typical apps.

## Feature flags

| Feature | Default | What it adds |
|---|---|---|
| `dal` | ✅ | `SqlUserRepository`, `AuthService` |
| `pq-seal` | ✅ | ML-KEM-768 envelope around stored hashes |

```toml
[dependencies]
ironroot-auth = "0.2"

# Hashing only, no DAL, no PQ:
ironroot-auth = { version = "0.2", default-features = false }
```

## Post-quantum design

**Argon2id is already post-quantum-secure** in the threat model that matters
for password hashing:

- No public-key cryptography is involved — Shor's algorithm does not apply.
- The symmetric primitives benefit at most from a square-root speedup under
  Grover's algorithm, which Argon2's memory-hardness parameters already
  absorb.

For an *additional* PQ layer, the `pq-seal` feature wraps the Argon2id PHC
string at rest:

1. The server holds an ML-KEM-768 keypair (NIST FIPS 203, ex-Kyber).
2. On store, encapsulate against the server's encapsulation key → derive a
   32-byte AEAD key with `SHA-256(shared_secret)`.
3. Encrypt the PHC bytes with ChaCha20-Poly1305 using a fresh 12-byte nonce.
4. The envelope `version || ct_len || ct || nonce || ciphertext` is encoded
   as URL-safe base64 (no padding) for storage in a plain `TEXT` column.

An attacker who exfiltrates only the database cannot brute-force the
Argon2id hashes offline — they'd also need the server's ML-KEM
decapsulation key.

## Quick start

```rust
use ironroot_auth::{
    AuthService, Credentials, NewUser, PasswordHasher,
    PqSealer, SqlUserRepository,
};
use ironroot_dal::Pool;
use std::sync::Arc;

# async fn run() -> Result<(), ironroot_auth::AuthError> {
let pool = Pool::connect("sqlite::memory:").await?;
let repo = SqlUserRepository::new(pool);
repo.ensure_schema().await?;

// Optional PQ sealing of stored hashes:
let sealer = Arc::new(PqSealer::generate());

let svc = AuthService::new(repo)
    .with_pq_sealer(sealer);

let user = svc.register(NewUser {
    username: "ada".into(),
    email: "ada@example.org".into(),
    password: "correct horse battery staple".into(),
}).await?;

let verified = svc.verify(&Credentials {
    username: "ada".into(),
    password: "correct horse battery staple".into(),
}).await?;
assert_eq!(verified.id, user.id);
# Ok(()) }
```

## Persisting the PQ keypair

`PqSealer::generate()` produces an ephemeral keypair. For real deployments,
persist `(dk_bytes, ek_bytes)` via `sealer.keypair_bytes()` and reload with
`PqSealer::from_keypair_bytes(&dk, &ek)`. Treat `dk_bytes` like any other
server secret (KMS, sealed file, etc.).

## Password validation

`NewUser::validate` enforces:

- Username: 3–64 chars, ASCII alphanumeric + `_`, `-`, `.`
- Email: contains `@`, length 3–320
- Password: 8–1024 chars

Customise by validating yourself before calling `register` and skipping
`validate`, or wrap `AuthService` in a higher-level service with your
project's policy.

## Error type

```rust
pub enum AuthError {
    InvalidCredentials,
    AlreadyExists,
    NotFound,
    Validation(String),
    Hash(String),
    #[cfg(feature = "pq-seal")] PqSeal(String),
    #[cfg(feature = "dal")]     Dal(ironroot_dal::DalError),
}
```

## See also

- [`ironroot-dal`](dal.md) — pool used by `SqlUserRepository`.
- [Argon2id RFC 9106](https://datatracker.ietf.org/doc/rfc9106/)
- [ML-KEM (NIST FIPS 203)](https://csrc.nist.gov/pubs/fips/203/final)
