# IronRoot — Architecture

## Overview

IronRoot is structured around a layered architecture that separates concerns into distinct crates. Each layer depends only on layers below it, keeping coupling low and testability high.

```
┌─────────────────────────────────────────┐
│            Application Layer            │
│  (templates: web-app / cli-app / gui)   │
├─────────────────────────────────────────┤
│            Interface Layer              │
│      crates/web  crates/cli  crates/gui │
├─────────────────────────────────────────┤
│          Infrastructure Layer           │
│          (future: db, cache, …)         │
├─────────────────────────────────────────┤
│            Domain / Core Layer          │
│              crates/core                │
├─────────────────────────────────────────┤
│            Cross-cutting               │
│             crates/macros               │
└─────────────────────────────────────────┘
```

---

## Crate Responsibilities

### `ironroot-core`

- Defines foundational traits (`Entity`, `Service`) used across all other crates.
- Intentionally lightweight — no async runtime, no I/O.
- OOP helpers (`classes` crate, `inherit-methods-macro`) may be used *optionally* inside this crate but must be clearly documented.

### `ironroot-macros`

- Procedural macro crate (`proc-macro = true`).
- Provides derive macros (`#[derive(Entity)]`) that implement `ironroot-core` traits automatically.
- All generated code must be readable and deterministic — no hidden side-effects.

### `ironroot-web`

- *(Planned)* HTTP server integration layer.
- Will expose traits for `Router`, `Middleware`, `Handler`, and `HttpContext`.
- Designed to be backend-agnostic (Axum, Actix-web, etc.) via trait adapters.

### `ironroot-cli`

- Provides `CliApp`, `Command`, and `CommandContext` abstractions.
- Thin wrapper intended to unify argument parsing libraries behind a common interface.

### `ironroot-gui`

- *(Planned)* Desktop UI integration.
- Will support Tauri (web-based) and/or egui (immediate-mode) via feature flags.

---

## Macro System

IronRoot macros follow these rules:

1. **Derive macros only** — avoid attribute macros unless necessary for ergonomics.
2. **Generate idiomatic code** — the expansion should look like handwritten Rust.
3. **Document expansion** — every macro must include a doc comment showing example input and its generated output.
4. **No `unsafe` in generated code** unless it cannot be avoided and is clearly justified.

---

## OOP Support Strategy

IronRoot takes a **traits-first** approach to object-oriented patterns:

1. **Traits** are the primary abstraction mechanism.
2. **Generics and associated types** encode polymorphism without virtual dispatch overhead.
3. **Dynamic dispatch (`dyn Trait`)** is used only at crate boundaries or when heterogeneous collections are required.
4. **Helper crates** (`classes`, `inherit-methods-macro`) provide ergonomic shortcuts for class-like patterns and method inheritance but are *optional extensions*, not core requirements.

Using OOP helpers must be justified and documented in the module where they appear.

---

## Dependency Rules

| Crate | May depend on |
|---|---|
| `ironroot-macros` | `proc-macro2`, `quote`, `syn` only |
| `ironroot-core` | `ironroot-macros` (optional) |
| `ironroot-web` | `ironroot-core` |
| `ironroot-cli` | `ironroot-core` |
| `ironroot-gui` | `ironroot-core` |
| Templates | Any of the above crates |

No crate in `crates/` may depend on a `templates/` project.
