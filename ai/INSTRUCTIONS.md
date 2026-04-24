# IronRoot — Extension & Contribution Instructions

This document explains how to extend IronRoot with new crates, macros, and templates, and describes the conventions every contributor (human or AI) must follow.

---

## How to Extend IronRoot

### Adding a New Crate

1. Create a directory under `crates/<name>/`.
2. Add a `Cargo.toml`:
   ```toml
   [package]
   name = "ironroot-<name>"
   version.workspace = true
   edition.workspace = true
   license.workspace = true
   ```
3. Add `"crates/<name>"` to the `members` list in the root `Cargo.toml`.
4. Create `src/lib.rs` with a module-level doc comment.
5. Add a section to `docs/architecture.md` describing the crate's responsibility.

### Adding a New Macro

1. Open `crates/macros/src/lib.rs`.
2. Add a new `#[proc_macro_derive]` (or `#[proc_macro_attribute]`) function.
3. Write the expansion using `proc-macro2`, `quote`, and `syn`.
4. Add an integration test in `crates/macros/tests/`.
5. Document the macro with input/output examples in its doc comment.

### Adding a New Template

1. Create a directory under `templates/<template-name>/`.
2. Initialise a standalone Cargo project (`cargo init --name <name>`).
3. Add a `README.md` explaining:
   - What the template demonstrates.
   - Which IronRoot crates it depends on.
   - How to build and run it.
4. Templates must **not** be added to the workspace `members` list — they are standalone projects.

---

## Naming Conventions

| Item | Convention | Example |
|---|---|---|
| Crate names | `ironroot-<domain>` (kebab-case) | `ironroot-core` |
| Trait names | `PascalCase`, noun or adjective | `Entity`, `Queryable` |
| Derive macros | `PascalCase` matching the trait | `#[derive(Entity)]` |
| Modules | `snake_case` | `pub mod user_service;` |
| Error types | `<Domain>Error` | `CoreError`, `WebError` |
| Feature flags | `kebab-case` | `features = ["async"]` |

---

## Code Organisation Rules

- **One concern per module.** A module must not mix domain logic with I/O.
- **`lib.rs` is a re-export hub.** Implementation lives in sub-modules; `lib.rs` only re-exports.
- **Keep `main.rs` thin.** Template `main.rs` files wire dependencies and call into library code.
- **No circular dependencies.** Dependency graph must be a DAG.

---

## Testing Expectations

| Layer | Test type |
|---|---|
| `ironroot-core` | Unit tests in `src/`, doc-tests |
| `ironroot-macros` | Integration tests in `crates/macros/tests/` using `trybuild` or `compiletest` |
| Interface crates | Unit tests + integration tests |
| Templates | `cargo build` and `cargo run` smoke tests in CI |

All tests must pass before merging. CI runs `cargo test --workspace`.

---

## Documentation Requirements

- Every public `struct`, `enum`, `trait`, `fn`, and `macro` must have a `///` doc comment.
- Modules must have a `//!` module-level comment.
- Non-obvious implementation details must have inline `//` comments.
- Architecture decisions must be recorded in `docs/architecture.md`.

---

## Guidelines for Introducing New Abstractions

Before adding a new trait, macro, or abstraction, ask:

1. **Does it already exist?** Check `ironroot-core` and the Rust standard library.
2. **Is it used in at least two places?** Abstractions that are only used once may be premature.
3. **Is the API stable?** Unstable APIs must be gated behind a `#[doc(hidden)]` or a `unstable` feature flag.
4. **Is it idiomatic?** The abstraction must feel natural to a Rust developer unfamiliar with IronRoot.
5. **Is it tested?** No abstraction without tests.

When in doubt, open a discussion issue before implementing.
