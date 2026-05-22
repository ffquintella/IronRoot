//! File-tree generation for a bootstrapped IronRoot project.

use std::fs;
use std::io;
use std::path::Path;

use crate::config::{Frontend, Gui, ProjectConfig, ProjectKind};

pub fn generate(cfg: &ProjectConfig, root: &Path) -> io::Result<()> {
    fs::create_dir_all(root)?;

    write(root, ".gitignore", &gitignore())?;
    write(root, "README.md", &readme(cfg))?;
    write(root, "Makefile", &makefile(cfg))?;
    write(root, "AGENTS.md", &agents_md(cfg))?;
    write(root, "rust-toolchain.toml", RUST_TOOLCHAIN)?;
    write(root, ".env.example", &env_example(cfg))?;

    match cfg.kind {
        ProjectKind::ClientTool => scaffold_cli(cfg, root)?,
        ProjectKind::WebApp => scaffold_webapp(cfg, root)?,
        ProjectKind::ClientServer => scaffold_client_server(cfg, root)?,
    }

    if let Some(frontend) = cfg.frontend {
        scaffold_frontend(frontend, root)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Scaffolds
// ---------------------------------------------------------------------------

fn scaffold_cli(cfg: &ProjectConfig, root: &Path) -> io::Result<()> {
    write(root, "Cargo.toml", &single_cargo_toml(cfg, "bin"))?;
    let src = root.join("src");
    fs::create_dir_all(&src)?;
    write(&src, "main.rs", &cli_main_rs(cfg))?;
    let tests = root.join("tests");
    fs::create_dir_all(&tests)?;
    write(&tests, "smoke.rs", CLI_SMOKE_TEST)?;
    scaffold_bdd(&tests, ProjectKind::ClientTool)?;
    Ok(())
}

fn scaffold_webapp(cfg: &ProjectConfig, root: &Path) -> io::Result<()> {
    write(root, "Cargo.toml", &single_cargo_toml(cfg, "bin"))?;
    let src = root.join("src");
    fs::create_dir_all(&src)?;
    write(&src, "main.rs", &webapp_main_rs(cfg))?;
    let tests = root.join("tests");
    fs::create_dir_all(&tests)?;
    write(&tests, "smoke.rs", WEBAPP_SMOKE_TEST)?;
    scaffold_bdd(&tests, ProjectKind::WebApp)?;
    Ok(())
}

fn scaffold_client_server(cfg: &ProjectConfig, root: &Path) -> io::Result<()> {
    // Workspace with `server/` and `client/` members.
    write(root, "Cargo.toml", &workspace_cargo_toml(cfg))?;

    // server
    let server = root.join("server");
    fs::create_dir_all(server.join("src"))?;
    fs::create_dir_all(server.join("tests"))?;
    write(&server, "Cargo.toml", &server_cargo_toml(cfg))?;
    write(&server.join("src"), "main.rs", &webapp_main_rs(cfg))?;
    write(&server.join("tests"), "smoke.rs", WEBAPP_SMOKE_TEST)?;
    scaffold_bdd(&server.join("tests"), ProjectKind::WebApp)?;

    // client
    let client = root.join("client");
    fs::create_dir_all(client.join("src"))?;
    fs::create_dir_all(client.join("tests"))?;
    write(&client, "Cargo.toml", &client_cargo_toml(cfg))?;
    let gui = cfg.gui.expect("client/server must have a gui");
    write(&client.join("src"), "main.rs", &client_main_rs(gui, &cfg.name))?;
    write(&client.join("tests"), "smoke.rs", CLIENT_SMOKE_TEST)?;
    scaffold_bdd(&client.join("tests"), ProjectKind::ClientServer)?;
    Ok(())
}

/// Drop a starter Gherkin feature file and step-definitions runner into
/// `tests/`. Cargo will pick `tests/bdd.rs` up automatically because each
/// generated `Cargo.toml` carries a matching `[[test]] harness = false` entry.
fn scaffold_bdd(tests_dir: &Path, _kind: ProjectKind) -> io::Result<()> {
    let features = tests_dir.join("features");
    fs::create_dir_all(&features)?;
    write(&features, "arithmetic.feature", BDD_FEATURE)?;
    write(tests_dir, "bdd.rs", BDD_RUNNER)?;
    Ok(())
}

fn scaffold_frontend(frontend: Frontend, root: &Path) -> io::Result<()> {
    let dir = root.join(frontend.dir_name());
    fs::create_dir_all(&dir)?;
    match frontend {
        Frontend::React => {
            write(&dir, "README.md", REACT_README)?;
            write(&dir, "package.json", REACT_PACKAGE_JSON)?;
            write(&dir, "index.html", REACT_INDEX_HTML)?;
            let src = dir.join("src");
            fs::create_dir_all(&src)?;
            write(&src, "main.tsx", REACT_MAIN_TSX)?;
            write(&src, "App.tsx", REACT_APP_TSX)?;
        }
        Frontend::Angular => {
            write(&dir, "README.md", ANGULAR_README)?;
            write(&dir, "package.json", ANGULAR_PACKAGE_JSON)?;
            let src = dir.join("src");
            fs::create_dir_all(&src)?;
            write(&src, "main.ts", ANGULAR_MAIN_TS)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Cargo.toml renderers
// ---------------------------------------------------------------------------

fn single_cargo_toml(cfg: &ProjectConfig, kind: &str) -> String {
    let mut deps = String::new();
    deps.push_str(BASE_DEPS);
    if matches!(cfg.kind, ProjectKind::WebApp) {
        deps.push_str(WEB_DEPS);
        deps.push_str(LOGGING_DEPS);
    } else if matches!(cfg.kind, ProjectKind::ClientTool) {
        deps.push_str(CLI_DEPS);
        deps.push_str(LOGGING_DEPS);
    }
    if let Some(feat) = cfg.database.sqlx_feature() {
        deps.push_str(&format!(
            "sqlx = {{ version = \"0.8\", features = [\"runtime-tokio-rustls\", \"{feat}\"] }}\n"
        ));
    }
    let bin_section = if kind == "bin" {
        format!("\n[[bin]]\nname = \"{}\"\npath = \"src/main.rs\"\n", cfg.name)
    } else {
        String::new()
    };
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
description = "Bootstrapped by ironroot"
{bin_section}
[dependencies]
{deps}
[dev-dependencies]
{bdd_deps}
{bdd_harness}"#,
        name = cfg.name,
        bdd_deps = BDD_DEV_DEPS,
        bdd_harness = BDD_HARNESS,
    )
}

fn workspace_cargo_toml(cfg: &ProjectConfig) -> String {
    format!(
        r#"[workspace]
resolver = "2"
members = ["server", "client"]

[workspace.package]
version = "0.1.0"
edition = "2021"
description = "Bootstrapped by ironroot ({name})"
"#,
        name = cfg.name,
    )
}

fn server_cargo_toml(cfg: &ProjectConfig) -> String {
    let mut deps = String::new();
    deps.push_str(BASE_DEPS);
    deps.push_str(WEB_DEPS);
    deps.push_str(LOGGING_DEPS);
    if let Some(feat) = cfg.database.sqlx_feature() {
        deps.push_str(&format!(
            "sqlx = {{ version = \"0.8\", features = [\"runtime-tokio-rustls\", \"{feat}\"] }}\n"
        ));
    }
    format!(
        r#"[package]
name = "{name}-server"
version.workspace = true
edition.workspace = true

[[bin]]
name = "{name}-server"
path = "src/main.rs"

[dependencies]
{deps}
[dev-dependencies]
{bdd_deps}
{bdd_harness}"#,
        name = cfg.name,
        bdd_deps = BDD_DEV_DEPS,
        bdd_harness = BDD_HARNESS,
    )
}

fn client_cargo_toml(cfg: &ProjectConfig) -> String {
    let gui = cfg.gui.expect("client/server must have a gui");
    let mut deps = String::new();
    deps.push_str(BASE_DEPS);
    match gui {
        Gui::Egui => {
            deps.push_str("eframe = \"0.34\"\negui = \"0.34\"\n");
        }
        Gui::Tauri => {
            deps.push_str("tauri = { version = \"2\", features = [] }\n");
        }
    }
    format!(
        r#"[package]
name = "{name}-client"
version.workspace = true
edition.workspace = true

[[bin]]
name = "{name}-client"
path = "src/main.rs"

[dependencies]
{deps}
[dev-dependencies]
{bdd_deps}
{bdd_harness}"#,
        name = cfg.name,
        bdd_deps = BDD_DEV_DEPS,
        bdd_harness = BDD_HARNESS,
    )
}

// ---------------------------------------------------------------------------
// Source file renderers
// ---------------------------------------------------------------------------

fn cli_main_rs(cfg: &ProjectConfig) -> String {
    format!(
        r#"//! `{name}` — CLI bootstrapped by ironroot.

{logging}
fn main() {{
    let _log_guard = init_logging("{name}");
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {{
        tracing::info!(version = env!("CARGO_PKG_VERSION"), "{name} starting");
        println!("{name} v{{}}", env!("CARGO_PKG_VERSION"));
        println!("usage: {name} <command> [args...]");
        return;
    }}
    match args[0].as_str() {{
        "hello" => {{
            tracing::info!(target = args.get(1).map(String::as_str).unwrap_or("world"), "greet");
            println!("hello, {{}}!", args.get(1).map(String::as_str).unwrap_or("world"));
        }}
        other => {{
            tracing::warn!(command = other, "unknown command");
            eprintln!("unknown command: {{other}}");
        }}
    }}
}}

pub fn add(a: i64, b: i64) -> i64 {{
    a + b
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn add_works() {{
        assert_eq!(add(2, 3), 5);
    }}
}}
"#,
        name = cfg.name,
        logging = LOGGING_BOOT,
    )
}

fn webapp_main_rs(cfg: &ProjectConfig) -> String {
    format!(
        r#"//! HTTP server bootstrapped by ironroot.

use axum::{{routing::get, Router}};

{logging}
#[tokio::main]
async fn main() {{
    let _log_guard = init_logging("{name}");

    let app = Router::new().route("/", get(root)).route("/health", get(health));

    let addr = "0.0.0.0:3000";
    tracing::info!("listening on {{addr}}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}}

async fn root() -> &'static str {{
    "ok"
}}

async fn health() -> &'static str {{
    "healthy"
}}

#[cfg(test)]
mod tests {{
    #[tokio::test]
    async fn health_returns_healthy() {{
        assert_eq!(super::health().await, "healthy");
    }}
}}
"#,
        name = cfg.name,
        logging = LOGGING_BOOT,
    )
}

fn client_main_rs(gui: Gui, name: &str) -> String {
    match gui {
        Gui::Egui => format!(
            r#"//! `{name}` desktop client (egui).

use eframe::egui;

fn main() -> Result<(), eframe::Error> {{
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "{name}",
        options,
        Box::new(|_cc| Box::<App>::default()),
    )
}}

#[derive(Default)]
struct App {{
    name: String,
}}

impl eframe::App for App {{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {{
        egui::CentralPanel::default().show(ctx, |ui| {{
            ui.heading("Hello from {name}");
            ui.horizontal(|ui| {{
                ui.label("Your name:");
                ui.text_edit_singleline(&mut self.name);
            }});
            if ui.button("Greet").clicked() {{
                println!("Hello, {{}}!", self.name);
            }}
        }});
    }}
}}
"#,
        ),
        Gui::Tauri => format!(
            r#"//! `{name}` desktop client (Tauri).
//!
//! NOTE: a real Tauri project requires `tauri.conf.json`, icons, and a
//! frontend build step. This file is a minimal placeholder; see
//! https://tauri.app for the full setup guide.

fn main() {{
    println!("This is a placeholder. Run `cargo tauri init` inside this crate");
    println!("to generate `tauri.conf.json`, then wire it up to the frontend/ dir.");
}}
"#,
        ),
    }
}

// ---------------------------------------------------------------------------
// Static file blobs
// ---------------------------------------------------------------------------

const RUST_TOOLCHAIN: &str = r#"[toolchain]
channel = "stable"
"#;

/// Baseline dependencies every generated project gets — error handling and
/// (de)serialization that almost every Rust app reaches for.
const BASE_DEPS: &str = "anyhow = \"1\"\n\
                         thiserror = \"2\"\n\
                         serde = { version = \"1\", features = [\"derive\"] }\n\
                         serde_json = \"1\"\n";

/// Extra deps for CLI-shaped projects.
const CLI_DEPS: &str = "clap = { version = \"4\", features = [\"derive\"] }\n";

/// Extra deps for HTTP-server-shaped projects.
const WEB_DEPS: &str = "axum = \"0.8\"\n\
                        tokio = { version = \"1\", features = [\"macros\", \"rt-multi-thread\", \"signal\"] }\n\
                        tower = \"0.5\"\n\
                        tower-http = { version = \"0.6\", features = [\"trace\", \"cors\"] }\n";

/// Behaviour-Driven Development dev-dependencies. `cucumber` runs Gherkin
/// `.feature` files; `tokio` is required because step functions are async.
const BDD_DEV_DEPS: &str = "cucumber = \"0.23\"\n\
                            tokio = { version = \"1\", features = [\"macros\", \"rt-multi-thread\"] }\n";

/// `[[test]]` section telling cargo to use the cucumber runner (no libtest
/// harness) for `tests/bdd.rs`.
const BDD_HARNESS: &str = "\n[[test]]\nname = \"bdd\"\nharness = false\n";

/// Starter Gherkin scenario. The matching step definitions live in
/// `tests/bdd.rs`. Add more `.feature` files alongside this one — the runner
/// picks up every file under `tests/features/`.
const BDD_FEATURE: &str = r#"Feature: Arithmetic helpers
  As a developer
  I want simple arithmetic helpers
  So that I can verify the BDD harness works end-to-end.

  Scenario: Adding two positive numbers
    Given I have the numbers 2 and 3
    When I add them together
    Then the result should be 5

  Scenario: Adding zero is identity
    Given I have the numbers 7 and 0
    When I add them together
    Then the result should be 7
"#;

/// Cucumber runner + step definitions. Lives at `tests/bdd.rs`. Extend the
/// `World` struct and add new `#[given]/#[when]/#[then]` functions as your
/// domain grows — keep them thin and delegate to real helpers in `src/`.
const BDD_RUNNER: &str = r#"//! BDD test runner — executes every `.feature` file under `tests/features/`.
//!
//! Run with `cargo test --test bdd` or `make bdd`.
//!
//! Step definitions should stay thin: parse Gherkin arguments, call into the
//! real business-logic helpers in `src/`, and assert on the result. Avoid
//! reimplementing logic inside steps — that defeats the purpose of BDD.

use cucumber::{given, then, when, World};

#[derive(Debug, Default, World)]
pub struct AppWorld {
    a: i64,
    b: i64,
    result: i64,
}

#[given(regex = r"^I have the numbers (-?\d+) and (-?\d+)$")]
async fn given_numbers(world: &mut AppWorld, a: i64, b: i64) {
    world.a = a;
    world.b = b;
}

#[when("I add them together")]
async fn when_added(world: &mut AppWorld) {
    // In a real project this should call into a helper from `src/`, e.g.
    // `world.result = my_crate::math::add(world.a, world.b);`
    world.result = world.a + world.b;
}

#[then(regex = r"^the result should be (-?\d+)$")]
async fn then_result(world: &mut AppWorld, expected: i64) {
    assert_eq!(world.result, expected);
}

#[tokio::main]
async fn main() {
    AppWorld::run("tests/features").await;
}
"#;


/// Dependency block giving generated projects working file-rotating logging
/// (10 MB rotation, 5 retained files) — mirrors `ironroot-log`'s defaults.
const LOGGING_DEPS: &str = "tracing = \"0.1\"\n\
                            tracing-subscriber = { version = \"0.3\", features = [\"env-filter\", \"fmt\"] }\n\
                            tracing-appender = \"0.2\"\n\
                            file-rotate = \"0.8\"\n";

/// Snippet inserted into generated `main.rs` files that boots the default
/// file-rotating logger. Mirrors `ironroot_log::init_default`.
const LOGGING_BOOT: &str = r#"// --- default rotating-file logger (10MB, keep 5) -----------------------
fn init_logging(app_name: &str) -> tracing_appender::non_blocking::WorkerGuard {
    use file_rotate::{compression::Compression, suffix::AppendCount, ContentLimit, FileRotate};
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    std::fs::create_dir_all("logs").expect("create logs dir");
    let path = std::path::Path::new("logs").join(format!("{app_name}.log"));
    let rotator = FileRotate::new(
        path,
        AppendCount::new(5),
        ContentLimit::Bytes(10 * 1024 * 1024),
        Compression::None,
        #[cfg(unix)]
        None,
    );
    let (writer, guard) = tracing_appender::non_blocking(rotator);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_ansi(false).with_writer(writer))
        .with(fmt::layer().with_writer(std::io::stdout))
        .init();
    guard
}
"#;

const CLI_SMOKE_TEST: &str = r#"//! Smoke test — the binary should build and the basic helper works.

#[test]
fn smoke() {
    assert_eq!(2 + 2, 4);
}
"#;

const WEBAPP_SMOKE_TEST: &str = r#"//! Smoke test placeholder for the HTTP server.

#[test]
fn smoke() {
    assert!(true, "replace with a real handler test");
}
"#;

const CLIENT_SMOKE_TEST: &str = r#"//! Smoke test placeholder for the desktop client.

#[test]
fn smoke() {
    assert!(true, "replace with a real UI/state test");
}
"#;

fn gitignore() -> String {
    r#"/target
**/*.rs.bk
Cargo.lock
.env
.DS_Store

# logs
logs/
*.log
*.log.*

# frontend
node_modules
dist
.vite
.angular
"#
    .to_string()
}

fn env_example(cfg: &ProjectConfig) -> String {
    let mut out = String::from("# Copy to `.env` and fill in real values.\n");
    if let Some(url) = cfg.database.default_url() {
        out.push_str(&format!("DATABASE_URL={url}\n"));
    }
    if matches!(cfg.kind, ProjectKind::WebApp | ProjectKind::ClientServer) {
        out.push_str("RUST_LOG=info\n");
        out.push_str("BIND_ADDR=0.0.0.0:3000\n");
    }
    out
}

fn makefile(cfg: &ProjectConfig) -> String {
    let has_frontend = cfg.frontend.is_some();
    let frontend_section = if has_frontend {
        r#"
.PHONY: frontend-install frontend-dev frontend-build
frontend-install:
	cd frontend && npm install
frontend-dev:
	cd frontend && npm run dev
frontend-build:
	cd frontend && npm run build
"#
    } else {
        ""
    };

    format!(
        r#".PHONY: build test bdd run fmt lint check clean

build:
	cargo build --workspace

test:
	cargo test --workspace

# Run BDD (Cucumber / Gherkin) scenarios only — feature files live under
# `tests/features/` and step definitions in `tests/bdd.rs`.
bdd:
	cargo test --workspace --test bdd

run:
	cargo run

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets -- -D warnings

check:
	cargo check --workspace --all-targets

clean:
	cargo clean
{frontend_section}"#,
    )
}

fn readme(cfg: &ProjectConfig) -> String {
    let mut layout = String::new();
    match cfg.kind {
        ProjectKind::ClientTool => layout.push_str("- `src/` — CLI entrypoint\n"),
        ProjectKind::WebApp => {
            layout.push_str("- `src/` — HTTP server entrypoint (axum)\n");
        }
        ProjectKind::ClientServer => {
            layout.push_str("- `server/` — HTTP server (axum)\n");
            layout.push_str("- `client/` — desktop GUI client\n");
        }
    }
    if cfg.frontend.is_some() {
        layout.push_str("- `frontend/` — JS/TS frontend\n");
    }

    format!(
        r#"# {name}

Bootstrapped by **ironroot**.

## Stack

- Project kind : {kind}
{gui_line}{frontend_line}- Database     : {db}

## Layout

{layout}
## Quickstart

```bash
make build
make test     # unit + integration + BDD
make bdd      # just the Gherkin scenarios
make run
```

## Testing

This project ships with two test layers:

- **Unit / integration tests** — standard `#[test]` functions in `src/` and
  `tests/`.
- **BDD scenarios** — Gherkin `.feature` files in `tests/features/`,
  executed by [cucumber-rs](https://crates.io/crates/cucumber). Step
  definitions live in `tests/bdd.rs`.

Add a new scenario by dropping a `.feature` file under `tests/features/`
and wiring matching `#[given]/#[when]/#[then]` steps into `tests/bdd.rs`.
See [AGENTS.md](AGENTS.md) for the full convention.

See [AGENTS.md](AGENTS.md) for AI-assistant guidance.
"#,
        name = cfg.name,
        kind = cfg.kind.label(),
        gui_line = cfg
            .gui
            .map(|g| format!("- GUI         : {}\n", g.label()))
            .unwrap_or_default(),
        frontend_line = cfg
            .frontend
            .map(|f| format!("- Frontend    : {}\n", f.label()))
            .unwrap_or_default(),
        db = cfg.database.label(),
    )
}

fn agents_md(cfg: &ProjectConfig) -> String {
    format!(
        r#"# AI Agent Instructions — {name}

This project was scaffolded by `ironroot`. AI assistants working on it
should follow the conventions below in addition to the upstream
[IronRoot AGENTS.md](https://github.com/ffquintella/IronRoot/blob/main/ai/AGENTS.md).

## Project shape

- Kind     : {kind}
{gui_line}{frontend_line}- Database : {db}

## House rules

1. **Prefer composition over inheritance** — use traits and generics; avoid
   deep type hierarchies.
2. **Keep `main.rs` thin** — wire dependencies and delegate to library code.
3. **One concern per module.** Domain logic and I/O do not mix.
4. **No silent breaking changes.** Bump versions, mark deprecations.
5. **Document every public item** with `///` doc comments.
6. **Tests are non-optional.** Every new feature ships with at least one test.
7. **`unsafe` is forbidden** unless justified inline with a `// SAFETY:` note.

## Business logic & helper modules

Business logic belongs in **small, focused helper modules** under `src/` —
never inlined into `main.rs`, handlers, or UI callbacks. The pattern:

- **One module per bounded concept.** `src/pricing.rs`, `src/auth.rs`,
  `src/billing/invoice.rs` — name the file after the *thing it owns*, not
  after a layer (`utils.rs`, `helpers.rs`, `common.rs` are anti-patterns;
  split them up).
- **Expose narrow types and free functions, not god-structs.** A helper like
  `pub fn calculate_tax(order: &Order) -> Money` is easier to test, reuse,
  and call from BDD steps than a sprawling `OrderService::do_everything`.
- **Group related helpers behind a trait** when there is more than one
  implementation (e.g. `trait Clock {{ fn now(&self) -> DateTime; }}` with a
  real and a `MockClock`). This is what makes the logic unit-testable and
  BDD-testable without I/O.
- **Inject dependencies; don't reach for globals.** Pass repositories,
  clocks, HTTP clients in as parameters or constructor args. Singletons and
  `lazy_static` make BDD scenarios non-deterministic.
- **Return rich errors.** Use `thiserror` to define a domain error per
  module (`PricingError`, `AuthError`) and let callers map them to HTTP /
  CLI / UI responses. Never `panic!` in business logic.
- **Pure first, side-effects at the edges.** If a helper *can* be a pure
  function of its inputs, make it one. Push DB / network / filesystem calls
  to thin adapters that the helper composes with — this is what lets the
  same helper power both `#[test]` and `#[when]` BDD steps.

When in doubt: if you cannot write a BDD scenario that exercises a helper
without spinning up a server or a database, the helper is doing too much.

## Testing

Two layers ship by default — use **both**:

1. **Unit & integration tests** (`#[test]` in `src/` and `tests/*.rs`) for
   fast, deterministic checks of individual helpers.
2. **BDD scenarios** (Gherkin `.feature` files under `tests/features/` with
   step definitions in `tests/bdd.rs`) for behaviour that crosses module
   boundaries or that a non-developer stakeholder should be able to read.

### Writing a BDD scenario

1. Add or extend a `.feature` file under `tests/features/`. Use
   `Given / When / Then` and write the *behaviour*, not the implementation:

   ```gherkin
   Feature: Invoice totals
     Scenario: VAT is applied to taxable line items
       Given an invoice with a 100.00 EUR taxable item
       When the totals are calculated
       Then the grand total should be 121.00 EUR
   ```

2. Add matching step functions in `tests/bdd.rs` (`#[given] / #[when] /
   #[then]`). Keep steps **thin** — parse arguments, call a helper from
   `src/`, assert. No business logic inside steps.
3. Extend `AppWorld` in `tests/bdd.rs` with whatever state the scenario
   needs to thread between steps.
4. Run `make bdd` (or `cargo test --test bdd`) locally before pushing.

## Workflow

- `make fmt`   — format the workspace.
- `make lint`  — run clippy with `-D warnings`.
- `make test`  — run the full test suite (unit + integration + BDD).
- `make bdd`   — run only the Cucumber/Gherkin scenarios.
- Update this file whenever a new convention is agreed.
"#,
        name = cfg.name,
        kind = cfg.kind.label(),
        gui_line = cfg
            .gui
            .map(|g| format!("- GUI      : {}\n", g.label()))
            .unwrap_or_default(),
        frontend_line = cfg
            .frontend
            .map(|f| format!("- Frontend : {}\n", f.label()))
            .unwrap_or_default(),
        db = cfg.database.label(),
    )
}

// --- frontend blobs --------------------------------------------------------

const REACT_README: &str = r#"# Frontend (React + Vite + TypeScript)

```bash
npm install
npm run dev      # start dev server
npm run build    # produce production bundle into dist/
```
"#;

const REACT_PACKAGE_JSON: &str = r#"{
  "name": "frontend",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "react": "^18.3.1",
    "react-dom": "^18.3.1"
  },
  "devDependencies": {
    "@types/react": "^18.3.3",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.1",
    "typescript": "^5.5.3",
    "vite": "^5.3.4"
  }
}
"#;

const REACT_INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>App</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"#;

const REACT_MAIN_TSX: &str = r#"import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
"#;

const REACT_APP_TSX: &str = r#"export default function App() {
  return <h1>Hello from React</h1>;
}
"#;

const ANGULAR_README: &str = r#"# Frontend (Angular)

```bash
npm install
npm start        # ng serve
npm run build    # production bundle
```

Run `npx ng new` if you prefer the official CLI scaffold instead of this
minimal placeholder.
"#;

const ANGULAR_PACKAGE_JSON: &str = r#"{
  "name": "frontend",
  "version": "0.1.0",
  "scripts": {
    "start": "ng serve",
    "build": "ng build",
    "test": "ng test"
  },
  "dependencies": {
    "@angular/animations": "^18.0.0",
    "@angular/common": "^18.0.0",
    "@angular/compiler": "^18.0.0",
    "@angular/core": "^18.0.0",
    "@angular/forms": "^18.0.0",
    "@angular/platform-browser": "^18.0.0",
    "@angular/platform-browser-dynamic": "^18.0.0",
    "@angular/router": "^18.0.0",
    "rxjs": "~7.8.0",
    "tslib": "^2.3.0",
    "zone.js": "~0.14.3"
  },
  "devDependencies": {
    "@angular/cli": "^18.0.0",
    "@angular/compiler-cli": "^18.0.0",
    "typescript": "~5.5.0"
  }
}
"#;

const ANGULAR_MAIN_TS: &str = r#"// Placeholder entrypoint. For a real app, generate one with `npx ng new`.
console.log("Hello from Angular");
"#;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn write(dir: &Path, name: &str, contents: &str) -> io::Result<()> {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)
}
