# IronRoot — AI Agents

This document defines the role of AI agents in the IronRoot project and the rules they must follow when contributing code, generating scaffolding, or assisting with reviews.

---

## Role of AI Agents

AI agents (GitHub Copilot, Claude, GPT-based models, etc.) are **first-class contributors** in IronRoot. They are expected to:

- Generate new crates, modules, and templates following existing conventions.
- Write documentation and doc-tests.
- Propose macro implementations.
- Assist in code review by identifying deviations from the rules below.
- Scaffold MCP server integrations.

Agents do **not** have authority to:

- Merge pull requests.
- Publish crates to crates.io.
- Change workspace-level dependency versions without human review.

---

## Rules

### 1. Prefer Composition Over Inheritance

Use traits and generic bounds to share behaviour. Avoid deep type hierarchies.

```rust
// Preferred
pub struct UserService<R: UserRepository> { repo: R }

// Avoid (unless justified and documented)
// class-based deep hierarchies via OOP helper crates
```

### 2. Use Traits and Macros Idiomatically

- Traits define interfaces; implementations live in the crate that owns the type.
- Derive macros reduce boilerplate but must generate readable, deterministic code.
- Attribute macros are a last resort — prefer derive macros or builder patterns.

### 3. Avoid `unsafe` Code Unless Required

If `unsafe` is necessary, it must be:

- Isolated in a dedicated module or crate.
- Justified with a `// SAFETY:` comment explaining the invariant.
- Covered by a test or property-based check.

### 4. OOP Helper Crates

The external [`classes`](https://crates.io/crates/classes) crate and
[`inherit-methods-macro`](https://crates.io/crates/inherit-methods-macro) crate enable
class-like programming patterns in Rust. They are **not bundled with IronRoot** — add
them to your own project's `[dependencies]` if needed. When used:

- **Document** every usage with a comment explaining *why* the OOP pattern is preferred over a plain trait.
- **Limit scope** — OOP helpers belong in implementation details, not public APIs.
- **Test thoroughly** — class hierarchies are harder to reason about; cover all public methods.

### 5. No Silent Breaking Changes

- Semver must be respected: breaking changes require a major version bump.
- Deprecated items must carry `#[deprecated]` with a migration note before removal.

### 6. Documentation Is Non-Optional

Every public item must have a doc comment. Agents generating new public APIs must include:

- A one-line summary.
- An example (`# Examples` section) where applicable.

---

## MCP Agent Integration

Agents operating via MCP (Model Context Protocol) must:

- Identify themselves in the `user-agent` field of MCP requests.
- Prefer read-only tools unless a write operation is explicitly requested.
- Log every action they perform to the MCP server's audit log.

See [mcp/README.md](../mcp/README.md) for server details.
