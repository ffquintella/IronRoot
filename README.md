# IronRoot

> A modular, full-stack Rust framework inspired by ASP.NET Zero — bringing structured, layered architecture and object-oriented patterns into idiomatic Rust.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](https://www.rust-lang.org/)

---

## Overview

IronRoot provides a foundation for building scalable, layered Rust applications with expressive abstractions and tooling support. It is designed for developers who want the productivity and structure of enterprise frameworks — without sacrificing the safety and performance that Rust delivers.

IronRoot is:

- **Modular** — pick only the crates you need.
- **Macro-driven** — reduce boilerplate with proc-macros.
- **OOP-friendly** — optional class-like abstractions via helper crates.
- **Full-stack** — templates for web, CLI, and desktop (GUI) applications.
- **AI-ready** — first-class support for AI agents and MCP (Model Context Protocol) integration.

---

## Key Features

| Feature | Description |
|---|---|
| Modular crate architecture | Each concern lives in its own crate under `crates/` |
| Macro-driven abstractions | Derive macros like `#[derive(Entity)]` reduce boilerplate |
| OOP-like support | Traits first; optional helpers (`classes`, `inherit-methods-macro`) for richer patterns |
| Multi-interface templates | Starter projects for web, CLI, and desktop applications |
| AI + MCP readiness | Built-in AI agent conventions and MCP server scaffolding |

---

## Project Structure

```
ironroot/
├── Cargo.toml              # Workspace root
├── README.md
├── .gitignore
├── docs/
│   ├── architecture.md     # Layered architecture overview
│   └── roadmap.md          # Development roadmap
├── ai/
│   ├── AGENTS.md           # AI agent roles and rules
│   └── INSTRUCTIONS.md     # Extension and contribution guidelines
├── mcp/
│   ├── servers/
│   │   └── example_server/ # Minimal MCP server binary
│   └── README.md
├── crates/
│   ├── core/               # Base traits: Entity, Service
│   ├── macros/             # Procedural macros (#[derive(Entity)], …)
│   ├── web/                # Web layer abstractions (future)
│   ├── cli/                # CLI abstraction layer
│   └── gui/                # Desktop UI placeholder (Tauri/egui)
└── templates/
    ├── web-app/            # Standalone web application template
    ├── cli-app/            # Standalone CLI application template
    └── desktop-app/        # Standalone desktop application template
```

---

## Getting Started

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable, 1.75+)
- `cargo` (included with Rust)

### Build the workspace

```bash
git clone https://github.com/ffquintella/IronRoot.git
cd IronRoot
cargo build
```

### Use a template

```bash
# Copy a template to start a new project
cp -r templates/cli-app my-cli-app
cd my-cli-app
cargo build && cargo run
```

---

## Crate Overview

### `ironroot-core`

Provides the foundational traits (`Entity`, `Service`) that all domain objects and services implement. This is the only mandatory dependency for IronRoot-based projects.

### `ironroot-macros`

Procedural macros (`#[derive(Entity)]`, etc.) that reduce boilerplate when implementing core traits.

### `ironroot-web`

*(Planned)* HTTP layer integration — routing, middleware, request/response abstractions.

### `ironroot-cli`

Provides abstractions for building command-line applications with structured argument parsing and command dispatch.

### `ironroot-gui`

*(Planned)* Desktop UI integration layer. Future support for Tauri and/or egui.

---

## Documentation

- [Architecture](docs/architecture.md) — layered design, crate responsibilities, macro system
- [Roadmap](docs/roadmap.md) — phased development plan
- [AI Agents](ai/AGENTS.md) — conventions for AI-assisted development
- [Extension Guide](ai/INSTRUCTIONS.md) — how to add crates, macros, and templates
- [MCP Integration](mcp/README.md) — Model Context Protocol server scaffolding

---

## Roadmap

See [docs/roadmap.md](docs/roadmap.md) for the full phased plan.

| Phase | Focus |
|---|---|
| 1 | Core abstractions (`core` crate) |
| 2 | Macro system (`macros` crate) |
| 3 | Templates (web / CLI / GUI) |
| 4 | MCP + AI agents |
| 5 | Ecosystem expansion |

---

## Contributing

Contributions are welcome! Please read the [extension guide](ai/INSTRUCTIONS.md) and follow the conventions described in [AGENTS.md](ai/AGENTS.md) before opening a pull request.

---

## License

[MIT](LICENSE)
