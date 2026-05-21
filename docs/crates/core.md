# `ironroot-core`

Foundational traits and abstractions used by every IronRoot crate.

## What's in it

- [`Entity`](#entity) — marker trait for domain objects with an identity.
- [`Service`](#service) — marker trait for application-layer services.

This crate is intentionally lightweight: no async runtime, no I/O. It is the
only **mandatory** dependency for IronRoot-based projects.

## Cargo

```toml
[dependencies]
ironroot-core = { path = "../path/to/IronRoot/crates/core" }   # or, once published
# ironroot-core = "0.2"
```

## `Entity`

A domain object that carries a stable identity. The trait is the contract
the macro derive in [`ironroot-macros`](macros.md) implements automatically.

```rust
use ironroot_core::Entity;

#[derive(Debug)]
pub struct User {
    pub id: u64,
    pub name: String,
}

impl Entity for User {
    type Id = u64;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}
```

With the derive macro:

```rust
use ironroot_macros::Entity;

#[derive(Debug, Entity)]
pub struct User {
    #[entity(id)]
    pub id: u64,
    pub name: String,
}
```

## `Service`

A marker trait for application-layer services. It carries no methods today
and exists to constrain generic bounds in downstream crates (e.g. when a web
handler accepts `S: Service` to inject application services into the
request lifecycle).

```rust
use ironroot_core::Service;

pub struct UserService;
impl Service for UserService {}
```

## Design notes

- **Composition over inheritance.** Implement traits on concrete types; do
  not reach for `classes`-style helpers unless an OOP pattern is genuinely
  the clearest expression. See [AGENTS.md](../../ai/AGENTS.md) §4.
- **No `unsafe`.** Core has zero `unsafe` blocks. If you ever need one,
  isolate it behind a documented `// SAFETY:` boundary and add a test.
- **Documentation is non-optional.** Every public item carries a `///` doc
  comment; missing one is a CI failure once `deny(missing_docs)` is enabled.
