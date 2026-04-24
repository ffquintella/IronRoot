# IronRoot — Roadmap

## Phase 1: Core Abstractions

**Goal:** Establish the foundational traits and project conventions.

- [x] Create workspace structure
- [x] Define `Entity` trait in `ironroot-core`
- [x] Define `Service` trait in `ironroot-core`
- [ ] Add documentation tests for core traits
- [ ] Publish initial `0.1.0` workspace to crates.io

---

## Phase 2: Macro System

**Goal:** Reduce boilerplate with procedural macros.

- [x] Scaffold `ironroot-macros` proc-macro crate
- [x] Add `#[derive(Entity)]` placeholder
- [ ] Implement full `Entity` derive (id field injection, trait impl)
- [ ] Add `#[derive(Service)]` derive macro
- [ ] Add integration tests for all macros
- [ ] Document generated code for each macro

---

## Phase 3: Templates (Web / CLI / GUI)

**Goal:** Provide ready-to-use starter projects for every target platform.

- [x] Scaffold `templates/web-app`
- [x] Scaffold `templates/cli-app`
- [x] Scaffold `templates/desktop-app`
- [ ] Implement real routing in `web-app` template (using `ironroot-web`)
- [ ] Implement command dispatch in `cli-app` template (using `ironroot-cli`)
- [ ] Integrate egui or Tauri into `desktop-app` template (using `ironroot-gui`)
- [ ] Add `cargo-generate` template manifests (`template.toml`)

---

## Phase 4: MCP + AI Agents

**Goal:** Enable AI-assisted development and orchestration via MCP.

- [x] Scaffold MCP example server (`mcp/servers/example_server`)
- [x] Define AI agent conventions (`ai/AGENTS.md`, `ai/INSTRUCTIONS.md`)
- [ ] Implement MCP protocol (stdio transport) in example server
- [ ] Expose IronRoot workspace tools as MCP resources
- [ ] Add agent prompts for generating new crates, macros, and templates
- [ ] Integrate with GitHub Copilot and/or Claude via MCP

---

## Phase 5: Ecosystem Expansion

**Goal:** Grow the framework into a complete application platform.

- [ ] `ironroot-db` — database abstraction layer (SQLx, SeaORM adapters)
- [ ] `ironroot-auth` — authentication and authorization helpers
- [ ] `ironroot-config` — layered configuration (env, file, secrets)
- [ ] `ironroot-events` — domain event bus
- [ ] `ironroot-testing` — test utilities and fixtures
- [ ] Plugin / extension system for third-party crates
- [ ] IronRoot CLI tool (`ironroot new`, `ironroot add`, …)
