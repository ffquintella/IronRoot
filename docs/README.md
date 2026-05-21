# IronRoot Documentation

> A modular, full-stack Rust framework inspired by ASP.NET Zero — bringing
> structured, layered architecture and object-oriented patterns into
> idiomatic Rust.

Welcome to the IronRoot documentation site. The content is plain Markdown
under [`docs/`](https://github.com/ffquintella/IronRoot/tree/main/docs) and
is rendered live by [docsify](https://docsify.js.org).

## Quick links

- [Architecture overview](architecture.md) — layered design, dependency rules
- [Roadmap](roadmap.md) — phased development plan
- **Crates** — one page per crate, see the sidebar
- [AI Agents guide](../ai/AGENTS.md) — conventions for AI-assisted development

## Reading the docs locally

```bash
# Unix / macOS
./docs/serve_docs.sh

# Windows
docs\serve_docs.bat
```

Both scripts try `docsify-cli` first (`npm i -g docsify-cli`) and fall back
to Python's `http.server` if Node is not available. Then open
<http://localhost:3000>.

## Crate index

| Crate | What it does |
|---|---|
| [`ironroot-core`](crates/core.md) | Foundational traits (`Entity`, `Service`) |
| [`ironroot-macros`](crates/macros.md) | Derive macros (`#[derive(Entity)]`) |
| [`ironroot-web`](crates/web.md) | HTTP layer abstractions *(planned)* |
| [`ironroot-cli`](crates/cli.md) | CLI app abstractions (`CliApp`, `Command`) |
| [`ironroot-gui`](crates/gui.md) | Desktop UI integration *(planned)* |
| [`ironroot-log`](crates/log.md) | Logging with 10 MB file rotation + optional syslog |
| [`ironroot-dal`](crates/dal.md) | Data Access Layer (SQLite + MySQL by default) |
| [`ironroot-dal-hiqlite`](crates/dal-hiqlite.md) | Hiqlite integration *(standalone crate)* |
| [`ironroot-auth`](crates/auth.md) | Argon2id passwords + optional ML-KEM PQ sealing |
| [`ironroot-new`](crates/new.md) | Interactive project bootstrapper |

## Project status

IronRoot is at an early stage; APIs are unstable until v0.x stabilises.
Contributions are welcome — see the
[extension guide](../ai/INSTRUCTIONS.md).
