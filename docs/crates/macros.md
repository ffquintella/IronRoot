# `ironroot-macros`

Procedural macros that reduce boilerplate when implementing
[`ironroot-core`](core.md) traits.

## What's in it

- `#[derive(Entity)]` — implements `ironroot_core::Entity` for a struct,
  picking the field marked `#[entity(id)]` as the identity, or falling back
  to a field literally named `id`.

## Cargo

```toml
[dependencies]
ironroot-core   = "0.2"
ironroot-macros = "0.2"
```

## Example

```rust
use ironroot_macros::Entity;

#[derive(Debug, Entity)]
pub struct User {
    #[entity(id)]
    pub user_id: u64,
    pub name: String,
    pub email: String,
}

let u = User { user_id: 42, name: "Ada".into(), email: "ada@example.org".into() };
assert_eq!(*u.id(), 42);
```

The derive expands to a small, deterministic `impl Entity for User { ... }`
block — open `cargo expand` (or your editor's inline expansion) if you want
to see exactly what is generated.

## Design rules

1. **Derive over attribute macros.** Attribute macros change item signatures
   in subtle ways; derives don't.
2. **Generated code must be readable.** No identifiers like `__ir_tmp1`; no
   hidden `unsafe`.
3. **Every macro ships with a doctest** in the macro's `///` comment showing
   the input and the generated output.
4. **`compile_error!` for clear diagnostics.** When the input is invalid,
   point at the offending span and explain the fix.
