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
    write(root, "CLAUDE.md", &claude_md(cfg))?;
    write(root, "CHANGELOG.md", &changelog_md(cfg))?;
    write(root, "rust-toolchain.toml", RUST_TOOLCHAIN)?;
    write(root, ".env.example", &env_example(cfg))?;
    scaffold_docs(cfg, root)?;
    scaffold_secure_development(root)?;

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
    write(root, "Cargo.toml", &single_cargo_toml(cfg))?;
    let src = root.join("src");
    fs::create_dir_all(&src)?;
    write(&src, "lib.rs", &cli_lib_rs(&cfg.name))?;
    write(&src, "logging.rs", LOGGING_RS)?;
    write(&src, "main.rs", &cli_main_rs(&cfg.name))?;
    let tests = root.join("tests");
    fs::create_dir_all(&tests)?;
    write(&tests, "smoke.rs", &cli_smoke_test(&cfg.name))?;
    scaffold_bdd(&tests, Layer::Cli, &cfg.name)?;
    Ok(())
}

fn scaffold_webapp(cfg: &ProjectConfig, root: &Path) -> io::Result<()> {
    write(root, "Cargo.toml", &single_cargo_toml(cfg))?;
    let src = root.join("src");
    fs::create_dir_all(&src)?;
    write(&src, "lib.rs", &server_lib_rs(&cfg.name))?;
    write(&src, "logging.rs", LOGGING_RS)?;
    write(&src, "main.rs", &server_main_rs(&cfg.name))?;
    let tests = root.join("tests");
    fs::create_dir_all(&tests)?;
    write(&tests, "smoke.rs", &server_smoke_test(&cfg.name))?;
    scaffold_bdd(&tests, Layer::Server, &cfg.name)?;
    Ok(())
}

fn scaffold_client_server(cfg: &ProjectConfig, root: &Path) -> io::Result<()> {
    // Workspace with `server/` and `client/` members.
    write(root, "Cargo.toml", &workspace_cargo_toml(cfg))?;

    // server
    let server_pkg = format!("{}-server", cfg.name);
    let server = root.join("server");
    fs::create_dir_all(server.join("src"))?;
    fs::create_dir_all(server.join("tests"))?;
    write(&server, "Cargo.toml", &server_cargo_toml(cfg))?;
    write(&server.join("src"), "lib.rs", &server_lib_rs(&server_pkg))?;
    write(&server.join("src"), "logging.rs", LOGGING_RS)?;
    write(&server.join("src"), "main.rs", &server_main_rs(&server_pkg))?;
    write(
        &server.join("tests"),
        "smoke.rs",
        &server_smoke_test(&server_pkg),
    )?;
    scaffold_bdd(&server.join("tests"), Layer::Server, &server_pkg)?;

    // client
    let client_pkg = format!("{}-client", cfg.name);
    let client = root.join("client");
    fs::create_dir_all(client.join("src"))?;
    fs::create_dir_all(client.join("tests"))?;
    write(&client, "Cargo.toml", &client_cargo_toml(cfg))?;
    let gui = cfg.gui.expect("client/server must have a gui");
    write(
        &client.join("src"),
        "lib.rs",
        &client_lib_rs(gui, &client_pkg),
    )?;
    write(
        &client.join("src"),
        "main.rs",
        &client_main_rs(gui, &client_pkg),
    )?;
    write(
        &client.join("tests"),
        "smoke.rs",
        &client_smoke_test(&client_pkg),
    )?;
    scaffold_bdd(&client.join("tests"), Layer::Client, &client_pkg)?;
    Ok(())
}

/// Which set of Gherkin scenarios and step definitions a crate gets. One per
/// shape of generated crate, because a scenario that does not drive the crate's
/// own library is a scenario that proves nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layer {
    Cli,
    Server,
    Client,
}

/// Drop the starter Gherkin feature file and its step definitions into
/// `tests/`. Cargo will pick `tests/bdd.rs` up automatically because each
/// generated `Cargo.toml` carries a matching `[[test]] harness = false` entry.
///
/// The steps call straight into the crate's `src/lib.rs`; they never
/// re-implement what they are meant to be checking.
fn scaffold_bdd(tests_dir: &Path, layer: Layer, package: &str) -> io::Result<()> {
    let features = tests_dir.join("features");
    fs::create_dir_all(&features)?;
    let (feature_name, feature, runner) = match layer {
        Layer::Cli => ("command-line.feature", CLI_FEATURE, CLI_BDD_RUNNER),
        Layer::Server => ("endpoints.feature", SERVER_FEATURE, SERVER_BDD_RUNNER),
        Layer::Client => ("greeting.feature", CLIENT_FEATURE, CLIENT_BDD_RUNNER),
    };
    write(&features, feature_name, &fill(feature, package))?;
    write(tests_dir, "bdd.rs", &fill(runner, package))?;
    Ok(())
}

/// Drop a docsify-backed `docs/` site into the project root, plus the
/// `serve-docs.sh` / `serve-docs.bat` helper launchers. The site renders the
/// project's own Markdown live (docsify-cli if available, else `http.server`).
fn scaffold_docs(cfg: &ProjectConfig, root: &Path) -> io::Result<()> {
    let docs = root.join("docs");
    fs::create_dir_all(&docs)?;
    write(&docs, ".nojekyll", "")?;
    write(&docs, "index.html", &docs_index_html(cfg))?;
    write(&docs, "README.md", &docs_readme(cfg))?;
    write(&docs, "_sidebar.md", DOCS_SIDEBAR)?;
    write(&docs, "_navbar.md", DOCS_NAVBAR)?;
    write(&docs, "getting-started.md", &docs_getting_started(cfg))?;
    write(&docs, "architecture.md", &docs_architecture(cfg))?;
    write(&docs, "roadmap.md", &docs_roadmap(cfg))?;
    write_exec(&docs, "serve-docs.sh", DOCS_SERVE_SH)?;
    write(&docs, "serve-docs.bat", DOCS_SERVE_BAT)?;
    Ok(())
}

/// Ship the secure-development rules and the coverage gates that back them:
///
/// - `.claude/skills/secure-development/SKILL.md` — `AGENTS.md` §5–§6 in the form an AI
///   assistant loads on its own. Claude Code discovers it automatically from the project root.
/// - `.security-sensitive` — the paths the 95% coverage floor applies to.
/// - `scripts/coverage-gate.py` — enforces 85% overall and 95% on those paths. It is a script
///   rather than a `cargo llvm-cov --fail-under-lines` flag because that flag can only express
///   one project-wide number.
fn scaffold_secure_development(root: &Path) -> io::Result<()> {
    let skill = root.join(".claude/skills/secure-development");
    fs::create_dir_all(&skill)?;
    write(&skill, "SKILL.md", SECURE_DEVELOPMENT_SKILL)?;

    write(root, ".security-sensitive", SECURITY_SENSITIVE)?;

    let scripts = root.join("scripts");
    fs::create_dir_all(&scripts)?;
    write_exec(&scripts, "coverage-gate.py", COVERAGE_GATE_PY)?;
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

fn single_cargo_toml(cfg: &ProjectConfig) -> String {
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
            "sqlx = {{ version = \"0.9\", features = [\"runtime-tokio\", \"tls-rustls\", \"{feat}\"] }}\n"
        ));
    }
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"
description = "Bootstrapped by ironroot"
{lib_section}
[[bin]]
name = "{name}"
path = "src/main.rs"

[dependencies]
{deps}
[dev-dependencies]
{bdd_deps}
{bdd_harness}"#,
        name = cfg.name,
        lib_section = lib_section(&cfg.name),
        bdd_deps = BDD_DEV_DEPS,
        bdd_harness = BDD_HARNESS,
    )
}

/// The `[lib]` target every generated crate carries.
///
/// It is not decoration. `src/main.rs` is compiled as a binary, and nothing in
/// a binary can be reached from `tests/*.rs` or from a cucumber step — so any
/// logic left there is logic the 85% coverage gate in `AGENTS.md` §4.3 counts
/// and no test can ever cover. The library target is what makes the two
/// mandatory test layers possible at all.
fn lib_section(package: &str) -> String {
    format!(
        "\n[lib]\nname = \"{crate_ident}\"\npath = \"src/lib.rs\"\n",
        crate_ident = crate_ident(package),
    )
}

/// The Rust identifier cargo derives from a package name: `my-app` -> `my_app`.
fn crate_ident(package: &str) -> String {
    package.replace('-', "_")
}

/// Expand the `{{package}}` / `{{crate}}` markers in a generated-source
/// template.
///
/// The templates are themselves Rust and Gherkin, both full of braces;
/// `format!` would need every one of them doubled, and a template nobody can
/// paste into a scratch file to check is a template that quietly drifts from
/// the code it is supposed to mirror. Markers keep them readable.
fn fill(template: &str, package: &str) -> String {
    template
        .replace("{{package}}", package)
        .replace("{{crate}}", &crate_ident(package))
}

fn workspace_cargo_toml(cfg: &ProjectConfig) -> String {
    format!(
        r#"[workspace]
resolver = "2"
members = ["server", "client"]

[workspace.package]
version = "0.1.0"
edition = "2024"
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
            "sqlx = {{ version = \"0.9\", features = [\"runtime-tokio\", \"tls-rustls\", \"{feat}\"] }}\n"
        ));
    }
    format!(
        r#"[package]
name = "{name}-server"
version.workspace = true
edition.workspace = true
{lib_section}
[[bin]]
name = "{name}-server"
path = "src/main.rs"

[dependencies]
{deps}
[dev-dependencies]
{bdd_deps}
{bdd_harness}"#,
        name = cfg.name,
        lib_section = lib_section(&format!("{}-server", cfg.name)),
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
            deps.push_str("eframe = \"0.36\"\negui = \"0.36\"\n");
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
{lib_section}
[[bin]]
name = "{name}-client"
path = "src/main.rs"

[dependencies]
{deps}
[dev-dependencies]
{bdd_deps}
{bdd_harness}"#,
        name = cfg.name,
        lib_section = lib_section(&format!("{}-client", cfg.name)),
        bdd_deps = BDD_DEV_DEPS,
        bdd_harness = BDD_HARNESS,
    )
}

// ---------------------------------------------------------------------------
// Source file renderers
// ---------------------------------------------------------------------------

fn cli_lib_rs(package: &str) -> String {
    fill(CLI_LIB_RS, package)
}

fn cli_main_rs(package: &str) -> String {
    fill(CLI_MAIN_RS, package)
}

fn cli_smoke_test(package: &str) -> String {
    fill(CLI_SMOKE_TEST, package)
}

fn server_lib_rs(package: &str) -> String {
    fill(SERVER_LIB_RS, package)
}

fn server_main_rs(package: &str) -> String {
    fill(SERVER_MAIN_RS, package)
}

fn server_smoke_test(package: &str) -> String {
    fill(SERVER_SMOKE_TEST, package)
}

/// The client's `src/lib.rs`: one toolkit-free core shared by both GUIs, plus
/// the shell for the chosen one. The core is where every decision lives, which
/// is what lets the unit tests and the scenarios drive the client without
/// opening a window.
///
/// The shell contributes its own tests to the same `mod tests`, so whatever a
/// backend adds to the file it also has to cover.
fn client_lib_rs(gui: Gui, package: &str) -> String {
    let (shell, shell_tests) = match gui {
        Gui::Egui => (EGUI_SHELL, EGUI_SHELL_TESTS),
        Gui::Tauri => (TAURI_SHELL, TAURI_SHELL_TESTS),
    };
    let source = [
        CLIENT_LIB_CORE,
        shell,
        CLIENT_LIB_TESTS,
        shell_tests,
        CLIENT_LIB_TESTS_END,
    ]
    .concat();
    fill(&source, package)
}

fn client_main_rs(gui: Gui, package: &str) -> String {
    let template = match gui {
        Gui::Egui => EGUI_MAIN_RS,
        Gui::Tauri => TAURI_MAIN_RS,
    };
    fill(template, package)
}

fn client_smoke_test(package: &str) -> String {
    fill(CLIENT_SMOKE_TEST, package)
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

/// Extra deps for HTTP-server-shaped projects. `tower`'s `util` feature carries
/// `ServiceExt::oneshot`, which is how the generated tests drive the router
/// in-process — no socket, no port, no flake.
const WEB_DEPS: &str = "axum = \"0.8\"\n\
                        tokio = { version = \"1\", features = [\"macros\", \"rt-multi-thread\", \"signal\"] }\n\
                        tower = { version = \"0.5\", features = [\"util\"] }\n\
                        tower-http = { version = \"0.7\", features = [\"trace\", \"cors\"] }\n";

/// Behaviour-Driven Development dev-dependencies. `cucumber` runs Gherkin
/// `.feature` files; `tokio` is required because step functions are async.
const BDD_DEV_DEPS: &str = "cucumber = \"0.23\"\n\
                            tokio = { version = \"1\", features = [\"macros\", \"rt-multi-thread\"] }\n";

/// `[[test]]` section telling cargo to use the cucumber runner (no libtest
/// harness) for `tests/bdd.rs`.
const BDD_HARNESS: &str = "\n[[test]]\nname = \"bdd\"\nharness = false\n";

/// Dependency block giving generated projects working file-rotating logging
/// (10 MB rotation, 5 retained files) — mirrors `ironroot-log`'s defaults.
const LOGGING_DEPS: &str = "tracing = \"0.1\"\n\
                            tracing-subscriber = { version = \"0.3\", features = [\"env-filter\", \"fmt\"] }\n\
                            tracing-appender = \"0.2\"\n\
                            file-rotate = \"0.8\"\n";

// --- generated `src/` --------------------------------------------------------
//
// Every template below is written so the generated project passes its own
// AGENTS.md §4 gate the moment it is created: the logic sits in a library
// target with unit tests beside it, `main.rs` only collects input and calls in,
// and the Gherkin scenarios drive the same library the unit tests do.
//
// The templates carry `{{package}}` (the cargo package name, e.g. `my-app`) and
// `{{crate}}` (its Rust identifier, `my_app`); `fill` substitutes both.

/// `src/logging.rs` — the single logging entry point required by `AGENTS.md`
/// §6.2, in a shape both test layers can reach. Identical in every generated
/// crate that logs, so it takes no substitutions.
const LOGGING_RS: &str = r##"//! The one logging entry point for this crate — AGENTS.md §6.2.
//!
//! One library (`tracing`), configured once, centrally. Every line goes to a
//! rotating file (10 MB per file, 5 kept) and to stdout. `main` calls [`init`]
//! once and holds the guard it returns for the life of the process — dropping
//! the guard is what flushes whatever the background writer still has buffered.
//!
//! The log directory and the filter spec are **parameters**, not globals read
//! from inside the functions. That is the same rule AGENTS.md applies to
//! business logic — inject the dependency — and it is what lets the tests at the
//! bottom of this file drive every branch, including the failures, without
//! writing into the working tree.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use file_rotate::compression::Compression;
use file_rotate::suffix::AppendCount;
use file_rotate::{ContentLimit, FileRotate};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

/// Bytes written to the current log file before it is rotated.
pub const ROTATE_BYTES: usize = 10 * 1024 * 1024;

/// How many rotated files are kept before the oldest is discarded.
pub const KEEP_FILES: usize = 5;

/// Filter used when `RUST_LOG` is unset or does not parse.
pub const DEFAULT_FILTER: &str = "info";

/// The file `app_name` logs to inside `dir`.
#[must_use]
pub fn log_path(dir: &Path, app_name: &str) -> PathBuf {
    dir.join(format!("{app_name}.log"))
}

/// The filter built from `spec`, falling back to [`DEFAULT_FILTER`].
///
/// A malformed `RUST_LOG` falls back rather than aborting: losing the logs is a
/// worse outcome than ignoring a typo in a filter.
#[must_use]
pub fn filter_from(spec: Option<&str>) -> EnvFilter {
    spec.and_then(|spec| EnvFilter::try_new(spec).ok())
        .unwrap_or_else(|| EnvFilter::new(DEFAULT_FILTER))
}

/// Installs the process-wide subscriber and returns its flush guard.
///
/// `dir` is created if it does not exist. Calling this a second time in one
/// process leaves the first subscriber in place — a second install is a wiring
/// mistake, not a reason to bring the process down.
///
/// # Errors
///
/// Returns the underlying [`io::Error`] when `dir` cannot be created.
pub fn init(dir: &Path, app_name: &str) -> io::Result<WorkerGuard> {
    let (writer, guard) = tracing_appender::non_blocking(rotating_file(dir, app_name)?);

    let installed = tracing_subscriber::registry()
        .with(filter_from(std::env::var("RUST_LOG").ok().as_deref()))
        .with(fmt::layer().with_ansi(false).with_writer(writer))
        .with(fmt::layer().with_writer(io::stdout))
        .try_init();
    if installed.is_err() {
        tracing::debug!("a tracing subscriber is already installed; keeping it");
    }

    Ok(guard)
}

/// Opens the rotating log file, creating `dir` first.
fn rotating_file(dir: &Path, app_name: &str) -> io::Result<FileRotate<AppendCount>> {
    fs::create_dir_all(dir)?;
    Ok(FileRotate::new(
        log_path(dir, app_name),
        AppendCount::new(KEEP_FILES),
        ContentLimit::Bytes(ROTATE_BYTES),
        Compression::None,
        #[cfg(unix)]
        None,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A directory of our own under the OS temp dir, named after the calling
    /// test so tests running in parallel never share one, and cleared first so
    /// each run starts from a known state.
    fn scratch(test: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "{}-{test}-{}",
            env!("CARGO_PKG_NAME"),
            std::process::id()
        ));
        let _unused = fs::remove_dir_all(&dir);
        let _unused = fs::remove_file(&dir);
        dir
    }

    #[test]
    fn the_log_file_is_named_after_the_application() {
        assert_eq!(
            log_path(Path::new("logs"), "my-app"),
            Path::new("logs").join("my-app.log")
        );
    }

    #[test]
    fn a_valid_filter_spec_is_used_as_given() {
        assert_eq!(filter_from(Some("warn")).to_string(), "warn");
    }

    #[test]
    fn no_filter_spec_falls_back_to_the_default() {
        assert_eq!(filter_from(None).to_string(), DEFAULT_FILTER);
    }

    #[test]
    fn a_malformed_filter_spec_falls_back_to_the_default() {
        assert_eq!(
            filter_from(Some("this=is=not=a=level")).to_string(),
            DEFAULT_FILTER
        );
    }

    #[test]
    fn init_creates_the_directory_and_tolerates_a_second_call() {
        let dir = scratch("init");
        assert!(!dir.exists(), "the scratch directory starts clean");

        let first = init(&dir, "test-app").expect("the first init installs the subscriber");
        assert!(dir.is_dir(), "init creates the log directory");

        let second = init(&dir, "test-app").expect("a second init is ignored, not an error");

        drop((first, second));
        let _unused = fs::remove_dir_all(&dir);
    }

    #[test]
    fn init_reports_a_log_directory_it_cannot_create() {
        let dir = scratch("blocked");
        fs::write(&dir, b"a file, not a directory").expect("the scratch path is writable");

        assert!(
            init(&dir, "test-app").is_err(),
            "a file standing where the log directory should be is an error, not a panic"
        );

        let _unused = fs::remove_file(&dir);
    }
}
"##;

/// `src/lib.rs` for a CLI project.
const CLI_LIB_RS: &str = r##"//! `{{package}}` — the library half of the CLI bootstrapped by ironroot.
//!
//! ## Why the logic lives here and not in `main`
//!
//! Every decision the binary makes lives in this library target so that both
//! test layers required by [`AGENTS.md`](../AGENTS.md) §4 can reach it:
//!
//! - unit tests in the `tests` module at the bottom of this file (§4.1),
//! - integration tests in `tests/smoke.rs` and cucumber scenarios in
//!   `tests/features/`, executed by `tests/bdd.rs` (§4.2).
//!
//! `src/main.rs` does one thing: collect the arguments and hand them to
//! [`respond`]. Keep it that way. Nothing in a binary target can be called from
//! a unit test or from a cucumber step, so logic left in `main` counts against
//! the 85% line-coverage gate in §4.3 with no way to cover it. It is also why
//! [`respond`] takes the arguments as a parameter instead of reading
//! [`std::env::args`] itself: a helper that reaches for process state cannot be
//! driven from a scenario.
//!
//! ## Growing this file
//!
//! One module per bounded concept — `src/pricing.rs`, `src/auth.rs` — not one
//! `utils.rs`. See AGENTS.md, "Business logic & helper modules".

pub mod logging;

/// The application name: the log file is named after it, and it opens the
/// usage text.
pub const APP_NAME: &str = "{{package}}";

/// Directory the rotating log files are written to, relative to the working
/// directory.
pub const LOG_DIR: &str = "logs";

/// Longest argument the CLI accepts, in characters.
///
/// A command line is untrusted input like any other, so it is bounded before it
/// is interpreted — AGENTS.md §5.3. Raise this deliberately; do not delete it.
pub const MAX_ARG_LEN: usize = 128;

/// Who `hello` greets when it is given no name.
pub const DEFAULT_GREETEE: &str = "world";

/// Why a command line was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CliError {
    /// An argument was longer than [`MAX_ARG_LEN`].
    #[error("argument {position} is {length} characters long; the maximum is {max}")]
    ArgumentTooLong {
        /// 1-based position of the offending argument, program name excluded.
        position: usize,
        /// Its length in characters.
        length: usize,
        /// The limit it exceeded.
        max: usize,
    },
    /// The first argument named a command that does not exist.
    #[error("unknown command: {verb}")]
    UnknownCommand {
        /// The word that was not recognised.
        verb: String,
    },
}

/// A command line that parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// No arguments — print the version and the usage text.
    Usage,
    /// `hello [name]` — greet `name`.
    Hello {
        /// Who to greet; [`DEFAULT_GREETEE`] when the argument was omitted.
        name: String,
    },
}

/// Parses `args` — the command line *after* the program name, exactly as
/// `std::env::args().skip(1)` yields it.
///
/// # Errors
///
/// [`CliError::ArgumentTooLong`] when any argument exceeds [`MAX_ARG_LEN`]; the
/// length is checked before the verb is looked at, so an oversized argument
/// never reaches a command. [`CliError::UnknownCommand`] when the first
/// argument is not a command this binary knows.
pub fn parse(args: &[String]) -> Result<Command, CliError> {
    for (index, arg) in args.iter().enumerate() {
        let length = arg.chars().count();
        if length > MAX_ARG_LEN {
            return Err(CliError::ArgumentTooLong {
                position: index + 1,
                length,
                max: MAX_ARG_LEN,
            });
        }
    }

    let Some(verb) = args.first() else {
        return Ok(Command::Usage);
    };

    match verb.as_str() {
        "hello" => Ok(Command::Hello {
            name: args
                .get(1)
                .map_or(DEFAULT_GREETEE, String::as_str)
                .to_owned(),
        }),
        other => Err(CliError::UnknownCommand {
            verb: other.to_owned(),
        }),
    }
}

/// Renders what a parsed `command` produces.
#[must_use]
pub fn render(command: &Command) -> String {
    match command {
        Command::Usage => usage(),
        Command::Hello { name } => format!("hello, {name}!"),
    }
}

/// The version banner and usage text shown when the CLI is given no arguments.
#[must_use]
pub fn usage() -> String {
    [
        format!("{APP_NAME} v{}", env!("CARGO_PKG_VERSION")),
        format!("usage: {APP_NAME} <command> [args...]"),
        String::new(),
        "commands:".to_owned(),
        format!("  hello [name]   greet [name], or {DEFAULT_GREETEE}"),
    ]
    .join("\n")
}

/// What the binary should write, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    text: String,
    failed: bool,
}

impl Response {
    /// The text to write.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Whether it belongs on stderr rather than stdout.
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.failed
    }

    /// The status the process should exit with: `0` on success, `1` on a
    /// refused command line.
    #[must_use]
    pub fn exit_status(&self) -> u8 {
        u8::from(self.failed)
    }
}

/// Handles a whole command line and reports what to print — the one function
/// `main` calls.
#[must_use]
pub fn respond(args: &[String]) -> Response {
    match parse(args) {
        Ok(command) => {
            tracing::info!(?command, "running command");
            Response {
                text: render(&command),
                failed: false,
            }
        }
        Err(error) => {
            tracing::warn!(%error, "refused command line");
            Response {
                text: error.to_string(),
                failed: true,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds an owned argument vector the way `main` collects one.
    fn argv(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn no_arguments_asks_for_the_usage_text() {
        assert_eq!(parse(&argv(&[])), Ok(Command::Usage));
    }

    #[test]
    fn hello_without_a_name_greets_the_world() {
        assert_eq!(
            parse(&argv(&["hello"])),
            Ok(Command::Hello {
                name: DEFAULT_GREETEE.to_owned()
            })
        );
    }

    #[test]
    fn hello_takes_the_name_that_follows_it() {
        assert_eq!(
            parse(&argv(&["hello", "Alice"])),
            Ok(Command::Hello {
                name: "Alice".to_owned()
            })
        );
    }

    #[test]
    fn an_unrecognised_verb_is_refused() {
        assert_eq!(
            parse(&argv(&["goodbye"])),
            Err(CliError::UnknownCommand {
                verb: "goodbye".to_owned()
            })
        );
    }

    #[test]
    fn an_argument_at_the_limit_is_accepted() {
        let at_limit = "a".repeat(MAX_ARG_LEN);
        assert_eq!(
            parse(&argv(&["hello", &at_limit])),
            Ok(Command::Hello { name: at_limit })
        );
    }

    #[test]
    fn an_argument_past_the_limit_never_reaches_a_command() {
        let too_long = "a".repeat(MAX_ARG_LEN + 1);
        assert_eq!(
            parse(&argv(&["hello", &too_long])),
            Err(CliError::ArgumentTooLong {
                position: 2,
                length: MAX_ARG_LEN + 1,
                max: MAX_ARG_LEN,
            })
        );
    }

    #[test]
    fn the_limit_counts_characters_and_not_bytes() {
        // `MAX_ARG_LEN` two-byte characters is exactly the limit, not twice it.
        let wide = "é".repeat(MAX_ARG_LEN);
        assert!(matches!(
            parse(&argv(&[&wide])),
            Err(CliError::UnknownCommand { .. })
        ));
    }

    #[test]
    fn the_usage_text_names_the_binary_and_its_commands() {
        let text = render(&Command::Usage);

        assert!(text.starts_with(APP_NAME));
        assert!(text.contains(&format!("usage: {APP_NAME} <command>")));
        assert!(text.contains("hello [name]"));
    }

    #[test]
    fn hello_renders_a_greeting() {
        assert_eq!(
            render(&Command::Hello {
                name: "Alice".to_owned()
            }),
            "hello, Alice!"
        );
    }

    #[test]
    fn a_command_line_that_parses_goes_to_stdout_and_exits_zero() {
        let response = respond(&argv(&["hello", "Alice"]));

        assert_eq!(response.text(), "hello, Alice!");
        assert!(!response.is_error());
        assert_eq!(response.exit_status(), 0);
    }

    #[test]
    fn a_refused_command_line_goes_to_stderr_and_exits_non_zero() {
        let response = respond(&argv(&["goodbye"]));

        assert_eq!(response.text(), "unknown command: goodbye");
        assert!(response.is_error());
        assert_eq!(response.exit_status(), 1);
    }

    #[test]
    fn the_length_error_says_which_argument_and_by_how_much() {
        let error = CliError::ArgumentTooLong {
            position: 2,
            length: 500,
            max: MAX_ARG_LEN,
        };

        assert_eq!(
            error.to_string(),
            format!("argument 2 is 500 characters long; the maximum is {MAX_ARG_LEN}")
        );
    }
}
"##;

/// `src/main.rs` for a CLI project — argument collection and one call in.
const CLI_MAIN_RS: &str = r##"//! Binary entry point for `{{package}}`.
//!
//! Deliberately thin: install the logger, collect the arguments, hand them to
//! [`{{crate}}::respond`], print what comes back. Everything worth testing lives
//! in `src/lib.rs` — see the module docs there for why nothing may move back
//! into this file.

use std::path::Path;
use std::process::ExitCode;

use {{crate}}::{APP_NAME, LOG_DIR, logging, respond};

fn main() -> Result<ExitCode, std::io::Error> {
    let _log_guard = logging::init(Path::new(LOG_DIR), APP_NAME)?;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let response = respond(&args);

    if response.is_error() {
        eprintln!("{}", response.text());
    } else {
        println!("{}", response.text());
    }

    Ok(ExitCode::from(response.exit_status()))
}
"##;

/// `tests/smoke.rs` for a CLI project.
const CLI_SMOKE_TEST: &str = r##"//! Integration tests — AGENTS.md §4.1, from outside the crate.
//!
//! `src/lib.rs` holds the unit tests and `tests/bdd.rs` the scenarios. This
//! file is the third view: it links `{{crate}}` as an external crate, so
//! everything it touches has to genuinely be `pub` and usable the way `main`
//! uses it.

use {{crate}}::{Command, DEFAULT_GREETEE, MAX_ARG_LEN, parse, respond};

fn argv(raw: &[&str]) -> Vec<String> {
    raw.iter().map(|arg| (*arg).to_owned()).collect()
}

#[test]
fn the_public_api_alone_is_enough_to_greet() {
    let response = respond(&argv(&["hello", "Alice"]));

    assert_eq!(response.text(), "hello, Alice!");
    assert!(!response.is_error());
    assert_eq!(response.exit_status(), 0);
}

#[test]
fn an_empty_command_line_explains_how_the_tool_is_invoked() {
    assert_eq!(parse(&argv(&[])), Ok(Command::Usage));

    let response = respond(&argv(&[]));

    assert!(response.text().contains("usage:"));
    assert!(response.text().contains(DEFAULT_GREETEE));
    assert!(!response.is_error());
}

#[test]
fn an_over_long_argument_is_refused_before_it_is_interpreted() {
    let response = respond(&argv(&["hello", &"a".repeat(MAX_ARG_LEN + 1)]));

    assert!(response.is_error());
    assert_eq!(response.exit_status(), 1);
    assert!(response.text().contains(&MAX_ARG_LEN.to_string()));
}
"##;

/// `tests/features/command-line.feature` for a CLI project.
const CLI_FEATURE: &str = r##"Feature: Command line handling

  `{{package}}` ships with one placeholder command. Until the real ones land it
  either greets, explains how it should be invoked, or refuses the command line
  and says why — it never silently does nothing.

  Scenario: hello greets the name it is given
    Given the command line "hello Alice"
    When the command line is handled
    Then the invocation succeeds
    And the output should contain "hello, Alice!"

  Scenario: hello with no name greets the world
    Given the command line "hello"
    When the command line is handled
    Then the invocation succeeds
    And the output should contain "hello, world!"

  Scenario: No arguments explain how the tool is invoked
    Given an empty command line
    When the command line is handled
    Then the invocation succeeds
    And the output should contain "usage:"
    And the output should contain "hello [name]"

  Scenario: An unknown command is refused
    Given the command line "goodbye"
    When the command line is handled
    Then the invocation fails
    And the output should contain "unknown command: goodbye"

  Scenario: An argument past the length limit is refused
    Given the command line "hello" followed by an argument of 500 characters
    When the command line is handled
    Then the invocation fails
    And the output should contain "the maximum is"
"##;

/// `tests/bdd.rs` for a CLI project.
const CLI_BDD_RUNNER: &str = r##"//! BDD runner — executes every `.feature` file under `tests/features/`.
//! AGENTS.md §4.2.
//!
//! Run with `make bdd`, or `cargo test --test bdd`.
//!
//! Steps stay thin: parse the Gherkin argument, call one helper from `src/`,
//! assert. Logic written inside a step is logic the unit tests and the coverage
//! gate never see — which is the one thing this layer exists to prevent.
//!
//! This target sets `harness = false` in `Cargo.toml` because cucumber brings
//! its own runner; `fn main` at the bottom is what `cargo test` executes.

use cucumber::{World, given, then, when};

use {{crate}}::{Response, respond};

/// State carried between the steps of a single scenario.
#[derive(Debug, Default, World)]
pub struct AppWorld {
    /// The command line, after the program name.
    args: Vec<String>,
    /// What the most recent `When` step produced.
    response: Option<Response>,
}

impl AppWorld {
    /// The response produced earlier in the scenario.
    ///
    /// # Panics
    ///
    /// Panics when no `When` step ran first — a wiring mistake in the feature
    /// file, which should fail loudly rather than assert against nothing.
    fn response(&self) -> &Response {
        self.response
            .as_ref()
            .expect("a `When` step must handle the command line before a `Then` step inspects it")
    }
}

#[given(expr = "the command line {string}")]
async fn given_command_line(world: &mut AppWorld, line: String) {
    world.args = line.split_whitespace().map(ToOwned::to_owned).collect();
}

#[given("an empty command line")]
async fn given_empty_command_line(world: &mut AppWorld) {
    world.args.clear();
}

#[given(expr = "the command line {string} followed by an argument of {int} characters")]
async fn given_padded_command_line(world: &mut AppWorld, verb: String, length: usize) {
    world.args = vec![verb, "a".repeat(length)];
}

#[when("the command line is handled")]
async fn when_handled(world: &mut AppWorld) {
    world.response = Some(respond(&world.args));
}

#[then("the invocation succeeds")]
async fn then_succeeds(world: &mut AppWorld) {
    let response = world.response();
    assert!(
        !response.is_error(),
        "expected success, got the error:\n{}",
        response.text()
    );
}

#[then("the invocation fails")]
async fn then_fails(world: &mut AppWorld) {
    let response = world.response();
    assert!(
        response.is_error(),
        "expected a refusal, got:\n{}",
        response.text()
    );
}

#[then(expr = "the output should contain {string}")]
async fn then_output_contains(world: &mut AppWorld, expected: String) {
    let text = world.response().text();
    assert!(
        text.contains(&expected),
        "expected the output to contain {expected:?}, got:\n{text}"
    );
}

#[then(expr = "the output should not contain {string}")]
async fn then_output_excludes(world: &mut AppWorld, forbidden: String) {
    let text = world.response().text();
    assert!(
        !text.contains(&forbidden),
        "expected the output not to contain {forbidden:?}, got:\n{text}"
    );
}

#[tokio::main]
async fn main() {
    AppWorld::run("tests/features").await;
}
"##;

/// `src/lib.rs` for an HTTP server, used both by the `webapp` kind and by the
/// `server/` member of a client/server workspace.
const SERVER_LIB_RS: &str = r##"//! `{{package}}` — the library half of the HTTP server bootstrapped by ironroot.
//!
//! ## Why the logic lives here and not in `main`
//!
//! The router and its handlers live in this library target so that both test
//! layers required by [`AGENTS.md`](../AGENTS.md) §4 can reach them:
//!
//! - unit tests in the `tests` module at the bottom of this file (§4.1),
//! - integration tests in `tests/smoke.rs` and cucumber scenarios in
//!   `tests/features/`, executed by `tests/bdd.rs` (§4.2).
//!
//! `src/main.rs` does one thing: read the environment and hand the router to
//! `axum::serve`. Keep it that way. Nothing in a binary target can be called
//! from a unit test or from a cucumber step, so a handler defined in `main` is a
//! handler that counts against the 85% line-coverage gate in §4.3 with no way to
//! cover it. Because [`router`] is a value rather than a running server, the
//! tests drive it in-process with `tower`'s `oneshot` — no socket, no port, no
//! flake.
//!
//! ## Growing this file
//!
//! Routes stay thin. Put the work in one module per bounded concept —
//! `src/pricing.rs`, `src/auth.rs` — and let the handler call it. See
//! AGENTS.md, "Business logic & helper modules".

pub mod logging;

use axum::Router;
use axum::routing::get;

/// The application name: the log file is named after it.
pub const APP_NAME: &str = "{{package}}";

/// Directory the rotating log files are written to, relative to the working
/// directory.
pub const LOG_DIR: &str = "logs";

/// Address the server binds when `BIND_ADDR` is unset or blank.
pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0:3000";

/// Builds the application router.
///
/// Register new routes here so every one of them is reachable from a test.
/// AGENTS.md §5.3 applies from the first route that takes input: bound it,
/// validate it server-side, and authorize through the single entry point.
pub fn router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
}

/// `GET /` — the placeholder index.
pub async fn root() -> &'static str {
    "ok"
}

/// `GET /health` — the liveness probe. Deliberately says nothing about the
/// process beyond "it answers": a health endpoint is unauthenticated, so it
/// leaks no version, hostname, or dependency state (AGENTS.md §5.3).
pub async fn health() -> &'static str {
    "healthy"
}

/// The address to bind, given the value of `BIND_ADDR`.
///
/// The environment is read by `main` and passed in, so every branch here is
/// testable and no test can be disturbed by the environment it runs in.
#[must_use]
pub fn bind_addr(configured: Option<&str>) -> String {
    match configured.map(str::trim) {
        Some(addr) if !addr.is_empty() => addr.to_owned(),
        _ => DEFAULT_BIND_ADDR.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    /// Drives `router()` in-process and returns the status and body of `path`.
    async fn get(path: &str) -> (StatusCode, String) {
        let request = Request::builder()
            .uri(path)
            .body(Body::empty())
            .expect("the test builds a valid request");
        let response = router()
            .oneshot(request)
            .await
            .expect("the router is infallible");

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("the test bodies are small");

        (
            status,
            String::from_utf8(body.to_vec()).expect("utf-8 body"),
        )
    }

    #[tokio::test]
    async fn the_index_answers_ok() {
        assert_eq!(get("/").await, (StatusCode::OK, "ok".to_owned()));
    }

    #[tokio::test]
    async fn the_health_probe_answers_healthy() {
        assert_eq!(get("/health").await, (StatusCode::OK, "healthy".to_owned()));
    }

    #[tokio::test]
    async fn the_health_probe_leaks_nothing_about_the_process() {
        let (_, body) = get("/health").await;

        assert!(!body.contains(env!("CARGO_PKG_VERSION")));
        assert_eq!(body.lines().count(), 1);
    }

    #[tokio::test]
    async fn an_unregistered_path_is_a_404() {
        let (status, _) = get("/does-not-exist").await;

        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn a_configured_bind_address_is_used_as_given() {
        assert_eq!(bind_addr(Some("127.0.0.1:8080")), "127.0.0.1:8080");
    }

    #[test]
    fn an_unset_bind_address_falls_back_to_the_default() {
        assert_eq!(bind_addr(None), DEFAULT_BIND_ADDR);
    }

    #[test]
    fn a_blank_bind_address_falls_back_rather_than_binding_nothing() {
        assert_eq!(bind_addr(Some("   ")), DEFAULT_BIND_ADDR);
    }

    #[test]
    fn surrounding_whitespace_is_trimmed_off_the_bind_address() {
        assert_eq!(bind_addr(Some("  0.0.0.0:9000 ")), "0.0.0.0:9000");
    }
}
"##;

/// `src/main.rs` for an HTTP server — environment, then one call in.
const SERVER_MAIN_RS: &str = r##"//! Binary entry point for `{{package}}`.
//!
//! Deliberately thin: install the logger, read the environment, bind, and serve
//! [`{{crate}}::router`]. Everything worth testing lives in `src/lib.rs` — see
//! the module docs there for why no handler may move back into this file.

use std::path::Path;

use {{crate}}::{APP_NAME, LOG_DIR, bind_addr, logging, router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _log_guard = logging::init(Path::new(LOG_DIR), APP_NAME)?;

    let addr = bind_addr(std::env::var("BIND_ADDR").ok().as_deref());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "listening");

    axum::serve(listener, router()).await?;

    Ok(())
}
"##;

/// `tests/smoke.rs` for an HTTP server.
const SERVER_SMOKE_TEST: &str = r##"//! Integration tests — AGENTS.md §4.1, from outside the crate.
//!
//! `src/lib.rs` holds the unit tests and `tests/bdd.rs` the scenarios. This
//! file is the third view: it links `{{crate}}` as an external crate and drives
//! the router the way a caller would, so everything it touches has to genuinely
//! be `pub`.

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use {{crate}}::{DEFAULT_BIND_ADDR, bind_addr, router};

#[tokio::test]
async fn every_registered_route_answers() {
    for (path, expected) in [("/", "ok"), ("/health", "healthy")] {
        let request = Request::builder()
            .uri(path)
            .body(Body::empty())
            .expect("the test builds a valid request");
        let response = router()
            .oneshot(request)
            .await
            .expect("the router is infallible");

        assert_eq!(response.status(), StatusCode::OK, "GET {path}");

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("the test bodies are small");
        assert_eq!(body.as_ref(), expected.as_bytes(), "GET {path}");
    }
}

#[test]
fn the_bind_address_is_configurable_and_has_a_safe_default() {
    assert_eq!(bind_addr(Some("127.0.0.1:0")), "127.0.0.1:0");
    assert_eq!(bind_addr(None), DEFAULT_BIND_ADDR);
}
"##;

/// `tests/features/endpoints.feature` for an HTTP server.
const SERVER_FEATURE: &str = r##"Feature: HTTP endpoints

  `{{package}}` answers two placeholder routes. They exist so the harness is
  wired end to end; replace them with the real ones and keep a scenario per
  route, negative cases included.

  Scenario: The index answers
    When a GET request is made to "/"
    Then the response status should be 200
    And the response body should be "ok"

  Scenario: The health probe answers
    When a GET request is made to "/health"
    Then the response status should be 200
    And the response body should be "healthy"

  Scenario: The health probe does not describe the process
    When a GET request is made to "/health"
    Then the response body should not contain "version"

  Scenario: An unregistered path is not found
    When a GET request is made to "/does-not-exist"
    Then the response status should be 404
"##;

/// `tests/bdd.rs` for an HTTP server.
const SERVER_BDD_RUNNER: &str = r##"//! BDD runner — executes every `.feature` file under `tests/features/`.
//! AGENTS.md §4.2.
//!
//! Run with `make bdd`, or `cargo test --test bdd`.
//!
//! Steps stay thin: parse the Gherkin argument, call one helper from `src/`,
//! assert. The router is driven in-process through `tower`'s `oneshot`, so a
//! scenario needs no port and cannot race another one.
//!
//! This target sets `harness = false` in `Cargo.toml` because cucumber brings
//! its own runner; `fn main` at the bottom is what `cargo test` executes.

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use cucumber::{World, then, when};
use tower::ServiceExt;

use {{crate}}::router;

/// State carried between the steps of a single scenario.
#[derive(Debug, Default, World)]
pub struct AppWorld {
    /// Status of the most recent response.
    status: Option<StatusCode>,
    /// Body of the most recent response.
    body: String,
}

impl AppWorld {
    /// The status of the response fetched earlier in the scenario.
    ///
    /// # Panics
    ///
    /// Panics when no `When` step ran first — a wiring mistake in the feature
    /// file, which should fail loudly rather than assert against nothing.
    fn status(&self) -> StatusCode {
        self.status
            .expect("a `When` step must make a request before a `Then` step inspects it")
    }
}

#[when(expr = "a GET request is made to {string}")]
async fn when_get(world: &mut AppWorld, path: String) {
    let request = Request::builder()
        .uri(&path)
        .body(Body::empty())
        .expect("the scenario supplies a valid path");
    let response = router()
        .oneshot(request)
        .await
        .expect("the router is infallible");

    world.status = Some(response.status());
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("the scenario bodies are small");
    world.body = String::from_utf8(body.to_vec()).expect("utf-8 body");
}

#[then(expr = "the response status should be {int}")]
async fn then_status(world: &mut AppWorld, expected: u16) {
    assert_eq!(world.status().as_u16(), expected);
}

#[then(expr = "the response body should be {string}")]
async fn then_body_is(world: &mut AppWorld, expected: String) {
    assert_eq!(world.body, expected);
}

#[then(expr = "the response body should not contain {string}")]
async fn then_body_excludes(world: &mut AppWorld, forbidden: String) {
    let body = world.body.to_lowercase();
    assert!(
        !body.contains(&forbidden.to_lowercase()),
        "expected the body not to contain {forbidden:?}, got:\n{body}"
    );
}

#[tokio::main]
async fn main() {
    AppWorld::run("tests/features").await;
}
"##;

/// The toolkit-free core of the desktop client's `src/lib.rs`, shared by both
/// GUI backends. Concatenated with a shell and the test module by
/// `client_lib_rs`.
const CLIENT_LIB_CORE: &str = r##"//! `{{package}}` — the library half of the desktop client bootstrapped by
//! ironroot.
//!
//! ## Why the logic lives here and not in `main`
//!
//! [`AppState`] holds no GUI type at all. That is deliberate, and it is what
//! lets both test layers required by [`AGENTS.md`](../AGENTS.md) §4 reach the
//! client's behaviour without opening a window:
//!
//! - unit tests in the `tests` module at the bottom of this file (§4.1),
//! - integration tests in `tests/smoke.rs` and cucumber scenarios in
//!   `tests/features/`, executed by `tests/bdd.rs` (§4.2).
//!
//! `src/main.rs` does one thing: call into this library. Keep it that way, and
//! keep the UI callbacks below just as thin. Nothing in a binary target — and
//! nothing that only runs from an event handler — can be called from a unit test
//! or from a cucumber step, so it counts against the 85% line-coverage gate in
//! §4.3 with no way to cover it.

/// Text shown at the top of the client.
pub const HEADING: &str = "Hello from {{package}}";

/// Longest name the greeter accepts, in characters.
///
/// A form field is untrusted input like any other, so it is bounded before it
/// is used — AGENTS.md §5.3. Raise this deliberately; do not delete it.
pub const MAX_NAME_LEN: usize = 64;

/// Why a name was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NameError {
    /// Nothing was typed, or only whitespace was.
    #[error("please type a name first")]
    Blank,
    /// The name was longer than [`MAX_NAME_LEN`].
    #[error("a name is at most {max} characters; this one is {length}")]
    TooLong {
        /// Its length in characters.
        length: usize,
        /// The limit it exceeded.
        max: usize,
    },
}

/// Everything the client knows, with no GUI type anywhere in it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AppState {
    name: String,
    message: String,
}

impl AppState {
    /// A client with nothing typed and nothing said yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The name currently typed in.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The buffer the text field writes into.
    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    /// The last thing the client had to say — empty until [`AppState::greet`]
    /// has run.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The greeting for the name currently typed in.
    ///
    /// # Errors
    ///
    /// [`NameError::Blank`] when nothing but whitespace was typed, and
    /// [`NameError::TooLong`] when the name exceeds [`MAX_NAME_LEN`].
    pub fn greeting(&self) -> Result<String, NameError> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(NameError::Blank);
        }

        let length = name.chars().count();
        if length > MAX_NAME_LEN {
            return Err(NameError::TooLong {
                length,
                max: MAX_NAME_LEN,
            });
        }

        Ok(format!("Hello, {name}!"))
    }

    /// Handles the greet action and stores what the client should display.
    ///
    /// A refusal is stored as a message too. A validation rule the user never
    /// sees fire is a rule they will keep tripping over.
    pub fn greet(&mut self) -> &str {
        self.message = match self.greeting() {
            Ok(greeting) => greeting,
            Err(error) => error.to_string(),
        };
        &self.message
    }
}
"##;

/// The egui shell appended to [`CLIENT_LIB_CORE`]: an [`eframe::App`] that only
/// moves values between the widgets and `AppState`.
const EGUI_SHELL: &str = r##"
/// The eframe shell. It owns an [`AppState`] and does nothing but move values
/// between the widgets and that state — every decision belongs above, where the
/// tests can reach it.
#[derive(Debug, Default)]
pub struct DesktopApp {
    state: AppState,
}

impl DesktopApp {
    /// The state behind the window.
    ///
    /// Exposed so the tests can assert on the client without opening one; the
    /// widgets below are the only other thing that touches it.
    #[must_use]
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Lays the client out into `ui`.
    ///
    /// Split out of the [`eframe::App`] impl deliberately: this takes only an
    /// `egui::Ui`, which `egui::__run_test_ui` can hand it headlessly, while the
    /// `eframe::Frame` the trait method also receives cannot be built outside a
    /// running window. That one parameter is the difference between a layout
    /// the tests cover and a layout only the coverage gate ever sees.
    ///
    /// Every line here moves a value. The moment one of them decides something,
    /// the decision belongs in [`AppState`].
    pub fn draw(&mut self, ui: &mut eframe::egui::Ui) {
        ui.heading(HEADING);
        ui.horizontal(|ui| {
            ui.label("Your name:");
            ui.text_edit_singleline(self.state.name_mut());
        });
        if ui.button("Greet").clicked() {
            self.state.greet();
        }
        if !self.state.message().is_empty() {
            ui.label(self.state.message());
        }
    }
}

impl eframe::App for DesktopApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }
}

/// Opens the window and runs until the user closes it.
///
/// # Errors
///
/// Propagates whatever eframe reports when the window cannot be created.
pub fn run() -> eframe::Result {
    eframe::run_native(
        "{{package}}",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::<DesktopApp>::default())),
    )
}
"##;

/// The Tauri shell appended to [`CLIENT_LIB_CORE`]. A real Tauri project needs
/// `tauri.conf.json`, icons, and a frontend build, so all this can honestly do
/// yet is say so — but it says it through a function the tests can call.
const TAURI_SHELL: &str = r##"
/// The placeholder banner the binary prints.
///
/// A real Tauri project needs `tauri.conf.json`, icons, and a frontend build
/// step, none of which this scaffold can invent. Run `cargo tauri init` inside
/// this crate and see <https://tauri.app> for the full setup.
///
/// [`AppState::greet`] above is already the shape a `#[tauri::command]` wants:
/// take the input, validate it, return `Result`. Wire it up as the first
/// command and the scenarios in `tests/features/` keep covering it unchanged.
#[must_use]
pub fn banner() -> String {
    [
        HEADING.to_owned(),
        "-".repeat(HEADING.chars().count()),
        "TODO: Open a native window using Tauri".to_owned(),
        "  1. cargo tauri init      (generates tauri.conf.json and icons)".to_owned(),
        "  2. expose AppState::greet as a #[tauri::command]".to_owned(),
        "  3. point the config at the frontend/ directory".to_owned(),
        String::new(),
        "See README.md for the full guide.".to_owned(),
    ]
    .join("\n")
}
"##;

/// The test module appended to the desktop client's `src/lib.rs`. It exercises
/// the toolkit-free core, which is all of the client's behaviour.
const CLIENT_LIB_TESTS: &str = r##"
#[cfg(test)]
mod tests {
    use super::*;

    /// A client with `name` already typed into the field.
    fn typed(name: &str) -> AppState {
        let mut state = AppState::new();
        state.name_mut().push_str(name);
        state
    }

    #[test]
    fn a_fresh_client_has_nothing_typed_and_nothing_to_say() {
        let state = AppState::new();

        assert_eq!(state.name(), "");
        assert_eq!(state.message(), "");
        assert_eq!(state, AppState::default());
    }

    #[test]
    fn the_heading_names_the_application() {
        assert!(HEADING.contains("{{package}}"));
    }

    #[test]
    fn a_name_is_greeted_by_name() {
        assert_eq!(typed("Alice").greeting(), Ok("Hello, Alice!".to_owned()));
    }

    #[test]
    fn surrounding_whitespace_is_trimmed_before_greeting() {
        assert_eq!(
            typed("  Alice \n").greeting(),
            Ok("Hello, Alice!".to_owned())
        );
    }

    #[test]
    fn an_empty_field_is_refused() {
        assert_eq!(typed("").greeting(), Err(NameError::Blank));
    }

    #[test]
    fn a_field_holding_only_whitespace_is_refused_too() {
        assert_eq!(typed("   \t ").greeting(), Err(NameError::Blank));
    }

    #[test]
    fn a_name_at_the_limit_is_accepted() {
        let at_limit = "a".repeat(MAX_NAME_LEN);

        assert_eq!(
            typed(&at_limit).greeting(),
            Ok(format!("Hello, {at_limit}!"))
        );
    }

    #[test]
    fn a_name_past_the_limit_is_refused() {
        assert_eq!(
            typed(&"a".repeat(MAX_NAME_LEN + 1)).greeting(),
            Err(NameError::TooLong {
                length: MAX_NAME_LEN + 1,
                max: MAX_NAME_LEN,
            })
        );
    }

    #[test]
    fn the_limit_counts_characters_and_not_bytes() {
        // `MAX_NAME_LEN` two-byte characters is exactly the limit, not twice it.
        let wide = "é".repeat(MAX_NAME_LEN);

        assert!(typed(&wide).greeting().is_ok());
    }

    #[test]
    fn greeting_stores_what_the_client_should_display() {
        let mut state = typed("Alice");

        assert_eq!(state.greet(), "Hello, Alice!");
        assert_eq!(state.message(), "Hello, Alice!");
    }

    #[test]
    fn a_refusal_is_displayed_rather_than_swallowed() {
        let mut state = typed("");

        assert_eq!(state.greet(), NameError::Blank.to_string());
        assert!(!state.message().is_empty());
    }
"##;

/// Backend-specific tests appended to [`CLIENT_LIB_TESTS`] for the egui shell.
const EGUI_SHELL_TESTS: &str = r##"
    #[test]
    fn the_window_starts_on_a_fresh_client() {
        assert_eq!(DesktopApp::default().state(), &AppState::new());
    }

    #[test]
    fn the_layout_draws_headlessly_and_decides_nothing() {
        let mut app = DesktopApp::default();

        // `__run_test_ui` is egui's own headless harness: a real `Ui`, no
        // window, no GPU. It is what keeps the layout inside the coverage gate.
        eframe::egui::__run_test_ui(|ui| app.draw(ui));

        assert_eq!(
            app.state(),
            &AppState::new(),
            "drawing moves values; it does not decide anything"
        );
    }

    #[test]
    fn the_layout_shows_whatever_the_client_last_said() {
        let mut app = DesktopApp::default();
        app.state.name_mut().push_str("Alice");
        app.state.greet();

        eframe::egui::__run_test_ui(|ui| app.draw(ui));

        assert_eq!(app.state().message(), "Hello, Alice!");
    }
"##;

/// Backend-specific tests appended to [`CLIENT_LIB_TESTS`] for the Tauri shell.
const TAURI_SHELL_TESTS: &str = r##"
    #[test]
    fn the_banner_underlines_the_heading() {
        let text = banner();
        let mut lines = text.lines();
        let title = lines.next().expect("the banner opens with the heading");
        let rule = lines.next().expect("the heading is underlined");

        assert_eq!(title, HEADING);
        assert_eq!(rule.chars().count(), title.chars().count());
        assert!(rule.chars().all(|dash| dash == '-'));
    }

    #[test]
    fn the_banner_says_what_is_still_missing() {
        let text = banner();

        assert!(text.contains("cargo tauri init"));
        assert!(text.contains("#[tauri::command]"));
    }

    #[test]
    fn the_banner_does_not_claim_a_window_was_opened() {
        let text = banner().to_lowercase();

        assert!(!text.contains("window opened"));
        assert!(text.contains("todo:"));
    }

    #[test]
    fn the_banner_is_one_block_without_a_trailing_newline() {
        let text = banner();

        assert!(!text.ends_with('\n'));
        assert!(text.lines().count() > 1);
    }
"##;

/// Closes the desktop client's `mod tests`, after the shared tests and whatever
/// the chosen backend added to them.
const CLIENT_LIB_TESTS_END: &str = r##"}
"##;

/// `src/main.rs` for an egui desktop client.
const EGUI_MAIN_RS: &str = r##"//! Binary entry point for the `{{package}}` desktop client (egui).
//!
//! Deliberately trivial: everything worth testing lives in [`{{crate}}`], the
//! library target next door. See `src/lib.rs` for the rationale and for the
//! TODO list that turns this scaffold into a real client.

fn main() -> eframe::Result {
    {{crate}}::run()
}
"##;

/// `src/main.rs` for a Tauri desktop client.
const TAURI_MAIN_RS: &str = r##"//! Binary entry point for the `{{package}}` desktop client (Tauri).
//!
//! Deliberately trivial: everything worth testing lives in [`{{crate}}`], the
//! library target next door. See `src/lib.rs` for the rationale and for the
//! TODO list that turns this scaffold into a real client.

fn main() {
    println!("{}", {{crate}}::banner());
}
"##;

/// `tests/smoke.rs` for a desktop client.
const CLIENT_SMOKE_TEST: &str = r##"//! Integration tests — AGENTS.md §4.1, from outside the crate.
//!
//! `src/lib.rs` holds the unit tests and `tests/bdd.rs` the scenarios. This
//! file is the third view: it links `{{crate}}` as an external crate, so
//! everything it touches has to genuinely be `pub`.
//!
//! Note what is *not* here: no window is opened. Every one of the client's
//! decisions lives in `AppState`, which is why this file can exist at all.

use {{crate}}::{AppState, HEADING, MAX_NAME_LEN, NameError};

#[test]
fn the_public_api_alone_is_enough_to_greet() {
    let mut state = AppState::new();
    state.name_mut().push_str("Alice");

    assert_eq!(state.greet(), "Hello, Alice!");
    assert_eq!(state.message(), "Hello, Alice!");
}

#[test]
fn the_heading_is_available_to_whatever_draws_it() {
    assert!(!HEADING.is_empty());
}

#[test]
fn an_over_long_name_is_refused_with_the_limit_it_broke() {
    let mut state = AppState::new();
    state.name_mut().push_str(&"a".repeat(MAX_NAME_LEN + 1));

    assert_eq!(
        state.greeting(),
        Err(NameError::TooLong {
            length: MAX_NAME_LEN + 1,
            max: MAX_NAME_LEN,
        })
    );
}
"##;

/// `tests/features/greeting.feature` for a desktop client.
const CLIENT_FEATURE: &str = r##"Feature: Greeting

  `{{package}}` greets whoever is named in its one input field. The field is
  untrusted input like any other, so the scenarios below cover what it refuses
  as carefully as what it accepts.

  Scenario: A name is greeted by name
    Given the name "Alice" is typed in
    When the client greets
    Then the client should say "Hello, Alice!"

  Scenario: Surrounding whitespace is ignored
    Given the name "  Alice  " is typed in
    When the client greets
    Then the client should say "Hello, Alice!"

  Scenario: An empty field is refused
    Given nothing is typed in
    When the client greets
    Then the client should say "please type a name first"

  Scenario: A name past the length limit is refused
    Given a name of 500 characters is typed in
    When the client greets
    Then the client should complain about the length
    And the client should not say "Hello"
"##;

/// `tests/bdd.rs` for a desktop client.
const CLIENT_BDD_RUNNER: &str = r##"//! BDD runner — executes every `.feature` file under `tests/features/`.
//! AGENTS.md §4.2.
//!
//! Run with `make bdd`, or `cargo test --test bdd`.
//!
//! Steps stay thin: parse the Gherkin argument, call one helper from `src/`,
//! assert. No window is opened — the scenarios drive `AppState`, which is the
//! whole reason the client's decisions live there and not in a UI callback.
//!
//! This target sets `harness = false` in `Cargo.toml` because cucumber brings
//! its own runner; `fn main` at the bottom is what `cargo test` executes.

use cucumber::{World, given, then, when};

use {{crate}}::{AppState, MAX_NAME_LEN};

/// State carried between the steps of a single scenario.
#[derive(Debug, Default, World)]
pub struct AppWorld {
    /// The client under test.
    client: AppState,
    /// What it said the last time it was asked to greet.
    said: Option<String>,
}

impl AppWorld {
    /// What the client said earlier in the scenario.
    ///
    /// # Panics
    ///
    /// Panics when no `When` step ran first — a wiring mistake in the feature
    /// file, which should fail loudly rather than assert against nothing.
    fn said(&self) -> &str {
        self.said
            .as_deref()
            .expect("a `When` step must greet before a `Then` step inspects what was said")
    }
}

#[given(expr = "the name {string} is typed in")]
async fn given_name(world: &mut AppWorld, name: String) {
    world.client.name_mut().push_str(&name);
}

#[given("nothing is typed in")]
async fn given_nothing(world: &mut AppWorld) {
    world.client.name_mut().clear();
}

#[given(expr = "a name of {int} characters is typed in")]
async fn given_long_name(world: &mut AppWorld, length: usize) {
    world.client.name_mut().push_str(&"a".repeat(length));
}

#[when("the client greets")]
async fn when_greeting(world: &mut AppWorld) {
    world.said = Some(world.client.greet().to_owned());
}

#[then(expr = "the client should say {string}")]
async fn then_says(world: &mut AppWorld, expected: String) {
    assert_eq!(world.said(), expected);
}

#[then(expr = "the client should not say {string}")]
async fn then_does_not_say(world: &mut AppWorld, forbidden: String) {
    let said = world.said();
    assert!(
        !said.contains(&forbidden),
        "expected {forbidden:?} not to be said, got:\n{said}"
    );
}

#[then("the client should complain about the length")]
async fn then_complains_about_length(world: &mut AppWorld) {
    let said = world.said();
    assert!(
        said.contains(&MAX_NAME_LEN.to_string()),
        "expected the refusal to name the {MAX_NAME_LEN}-character limit, got:\n{said}"
    );
}

#[tokio::main]
async fn main() {
    AppWorld::run("tests/features").await;
}
"##;

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
    // Read by `logging::init`, which every generated binary that logs calls
    // from `main`. An unparseable value falls back to `info` rather than
    // leaving the process with no logs at all.
    out.push_str("RUST_LOG=info\n");
    if matches!(cfg.kind, ProjectKind::WebApp | ProjectKind::ClientServer) {
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
        r#".PHONY: build test bdd coverage audit run fmt lint check clean docs

build:
	cargo build --workspace

test:
	cargo test --workspace

# Run BDD (Cucumber / Gherkin) scenarios only — feature files live under
# `tests/features/` and step definitions in `tests/bdd.rs`.
bdd:
	cargo test --workspace --test bdd

# Both coverage floors from AGENTS.md §4.3: 85% of lines overall, and 95% on every
# path listed in `.security-sensitive`. Requires: cargo install cargo-llvm-cov
coverage:
	./scripts/coverage-gate.py

# Supply-chain audit. Requires: cargo install cargo-audit cargo-deny
audit:
	cargo audit
	cargo deny check

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

# Serve the docsify documentation site on http://localhost:3000
docs:
	./docs/serve-docs.sh
{frontend_section}"#,
    )
}

fn readme(cfg: &ProjectConfig) -> String {
    let mut layout = String::new();
    match cfg.kind {
        ProjectKind::ClientTool => {
            layout.push_str("- `src/lib.rs` — the CLI's logic, and the unit tests for it\n");
            layout.push_str("- `src/logging.rs` — the single logging entry point\n");
            layout.push_str("- `src/main.rs` — collects the arguments, calls the library\n");
        }
        ProjectKind::WebApp => {
            layout
                .push_str("- `src/lib.rs` — the router and its handlers, with their unit tests\n");
            layout.push_str("- `src/logging.rs` — the single logging entry point\n");
            layout.push_str("- `src/main.rs` — reads the environment, binds, serves the router\n");
        }
        ProjectKind::ClientServer => {
            layout.push_str(
                "- `server/` — HTTP server (axum); logic in `src/lib.rs`, tests in `tests/`\n",
            );
            layout.push_str(
                "- `client/` — desktop GUI client; logic in `src/lib.rs`, tests in `tests/`\n",
            );
        }
    }
    if !matches!(cfg.kind, ProjectKind::ClientServer) {
        layout.push_str(
            "- `tests/` — integration tests (`smoke.rs`) and BDD (`features/`, `bdd.rs`)\n",
        );
    }
    if cfg.frontend.is_some() {
        layout.push_str("- `frontend/` — JS/TS frontend\n");
    }
    layout.push_str("- `docs/` — docsify documentation site (`make docs`)\n");
    layout.push_str("- `scripts/coverage-gate.py` — the 85% / 95% coverage gates\n");
    layout.push_str("- `.security-sensitive` — paths the 95% coverage floor applies to\n");
    layout.push_str("- `.claude/skills/` — the secure-development skill, loaded automatically\n");

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
make coverage # coverage gates: 85% of lines overall, 95% security-sensitive
make audit    # cargo audit + cargo deny check
make run
make docs     # serve the docsify docs on http://localhost:3000
```

## Testing

This project ships with two test layers, both passing and both above the coverage floor
before you write a line:

- **Unit / integration tests** — `#[test]` functions in `src/` (`mod tests`) and in
  `tests/smoke.rs`.
- **BDD scenarios** — Gherkin `.feature` files in `tests/features/`,
  executed by [cucumber-rs](https://crates.io/crates/cucumber). Step
  definitions live in `tests/bdd.rs`.

Add a new scenario by dropping a `.feature` file under `tests/features/`
and wiring matching `#[given]/#[when]/#[then]` steps into `tests/bdd.rs`.

**Write the code in `src/lib.rs`, not in `src/main.rs`.** Nothing in a binary target can be
called from a unit test or from a cucumber step, so a function defined in `main` can never be
covered — it can only count against the gate. `main.rs` collects input and calls the library;
that is its whole job. The same applies to a UI callback or an HTTP handler: keep the shell
thin and put the decision in a helper the tests can call.

Coverage is gated twice by `make coverage`: **85%** of lines overall, and **95%** on every path
listed in [`.security-sensitive`](.security-sensitive) — see [AGENTS.md](AGENTS.md) §4.3.

## Secure development

[`.claude/skills/secure-development/SKILL.md`](.claude/skills/secure-development/SKILL.md) ships
with this project. Claude Code loads it automatically; point other assistants at it. It carries
[AGENTS.md](AGENTS.md) §5–§6 in working form — untrusted input, parameterized queries, one
authentication entry point, secrets, error handling, the audit trail, dependency hygiene — each
with the Rust pattern that satisfies it, and the checklist to run before calling a change done.

## House rules

[AGENTS.md](AGENTS.md) is the single source of truth for how work is done here:
roadmap-driven planning, semantic versioning, changelog upkeep, unit **and** behaviour tests
above 85% coverage — 95% on security-sensitive paths — secure-coding requirements, and full
audit coverage. [CLAUDE.md](CLAUDE.md) points Claude Code at the same file, and the
[`secure-development` skill](.claude/skills/secure-development/SKILL.md) carries §5–§6 in the form
an AI assistant loads on its own.

Record every user-visible change in [CHANGELOG.md](CHANGELOG.md) under `## [Unreleased]`, in
the same commit that makes the change. Plan it in [docs/roadmap.md](docs/roadmap.md) first.
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
2. **Keep `main.rs` thin** — collect the input, call `src/lib.rs`, print the
   result. Nothing in a binary target is reachable from a unit test or a
   cucumber step, so logic left in `main` is logic no test can cover and the
   §4.3 gate counts anyway. The same applies to UI callbacks and HTTP
   handlers: the shell moves values, the helper decides.
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

Two layers ship by default and **both** are mandatory — unit/integration tests
plus BDD scenarios, with line coverage held above 85% overall and above 95% on
security-sensitive files. The policy is in §4
below; this section is the mechanical walkthrough.

Both layers ship **passing**, above the floor, on a freshly generated project.
Keep them that way: the gate is a floor you stay above, not a milestone you
reach later.

### Where the code goes

`src/lib.rs` — not `src/main.rs`. A binary target is not linked by
`tests/*.rs` and not reachable from a cucumber step, so anything defined in
`main` cannot be exercised by either mandatory layer while still counting
against the §4.3 line-coverage floor. `main.rs` collects the input and calls
into the library; that is all it ever does. The same reasoning applies inside
the library: an HTTP handler or a UI callback moves values, and the decision it
would otherwise make belongs in a helper the tests can call directly.

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

- `make fmt`      — format the workspace.
- `make lint`     — run clippy with `-D warnings`.
- `make test`     — run the full test suite (unit + integration + BDD).
- `make bdd`      — run only the Cucumber/Gherkin scenarios.
- `make coverage` — both coverage floors: 85% overall, 95% security-sensitive.
- `make audit`    — `cargo audit` + `cargo deny check`.
- Update this file whenever a new convention is agreed.
{rules}"#,
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
        rules = AGENT_RULES,
    )
}

fn claude_md(cfg: &ProjectConfig) -> String {
    format!(
        r#"# CLAUDE.md — {name}

**All project instructions live in [AGENTS.md](AGENTS.md). Read that file first; it is the
single source of truth.** This file exists only so that Claude Code picks the rules up
automatically — it deliberately duplicates nothing.

> @AGENTS.md

The secure-development rules in AGENTS.md §5–§6 also ship as a skill you load automatically:
[`.claude/skills/secure-development/SKILL.md`](.claude/skills/secure-development/SKILL.md). Use it
whenever a change touches authentication, authorization, secrets, queries, untrusted input, error
handling, logging, the audit trail, or a dependency — and before calling any change done.

## Quick reminders (the full rules are in AGENTS.md)

| Topic | Rule | Section |
|---|---|---|
| Roadmap | Every change maps to an item in [`docs/roadmap.md`](docs/roadmap.md); tick it in the same commit | §1 |
| Versioning | Semantic Versioning; no silent breaking changes; tag every release | §2 |
| Changelog | Update [`CHANGELOG.md`](CHANGELOG.md) under `## [Unreleased]` in the same commit | §3 |
| Tests | Unit/integration **and** BDD scenarios — both, every feature | §4 |
| Coverage | `make coverage` must pass — **85%** of lines overall, **95%** on every file in [`.security-sensitive`](.security-sensitive) — and coverage must not drop | §4.3 |
| Secure code | Parameterized queries, output escaping, bounded input, handled errors, no secrets in the repo, no `unsafe` | §5 |
| Audit | Every security-relevant action audited to an INSERT-only store on a separate instance; `make audit` clean | §6 |
| Done | Work through the checklist before saying a change is finished | §7 |

## Before you report a change as complete

```bash
make fmt
make lint
make test
make coverage
make audit
```

Do not report work as done until these pass and the AGENTS.md §7 checklist is satisfied.
"#,
        name = cfg.name,
    )
}

fn changelog_md(cfg: &ProjectConfig) -> String {
    format!(
        r#"# Changelog

All notable changes to `{name}` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0/).

Every user-visible change gets an entry under `## [Unreleased]` **in the same commit that
makes the change** — see [AGENTS.md](AGENTS.md) §3.

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.1.0] - Unreleased

### Added

- Project bootstrapped by `ironroot` ({kind}).
- Library target `src/lib.rs` holding the logic, with `src/main.rs` reduced to
  collecting input and calling into it — the split both mandatory test layers
  need, since nothing in a binary target is reachable from `tests/*.rs` or from
  a cucumber step (AGENTS.md §4).
- Unit tests, integration tests in `tests/smoke.rs`, and Gherkin scenarios in
  `tests/features/` driving that library. `make coverage` passes as generated.
"#,
        name = cfg.name,
        kind = cfg.kind.label(),
    )
}

/// The project-independent half of `AGENTS.md`: roadmap discipline, semver,
/// changelog upkeep, the two test layers and the 85%/95% coverage gates, secure
/// coding requirements, and audit coverage. Kept as a plain `const` (rather
/// than folded into the `format!` above) so its many `{}`-free code samples
/// need no brace escaping.
const AGENT_RULES: &str = r##"
---

**The numbered sections below are binding.** Section 7 is the checklist to run before
calling any change complete.

## 1. Follow the roadmap

Work is roadmap-driven. Before starting anything:

1. Read [`docs/roadmap.md`](docs/roadmap.md) and find the phase the task belongs to.
2. If the task is **not** on the roadmap, add it there first (as an unchecked item under the
   right phase) and say so in the pull request. Do not silently widen scope.
3. Tick the roadmap checkbox in the **same** commit that lands the work — never ahead of it.
4. Do not start a later phase while an earlier phase has open items that the task depends on.

Roadmap items are the unit of planning; changelog entries are the unit of record. Every
completed roadmap item produces at least one changelog entry.

---

## 2. Semantic versioning

This project follows [Semantic Versioning 2.0.0](https://semver.org/) — `MAJOR.MINOR.PATCH`.

| Change | Bump |
|---|---|
| Removing or renaming a public item; changing a signature, trait bound, or serialized format; tightening validation that rejects previously accepted input | **MAJOR** |
| New public item, new feature flag, new optional config, new endpoint or command | **MINOR** |
| Bug fix, performance work, docs, internal refactor with no public surface change | **PATCH** |
| Dependency bump | **PATCH**, unless it changes this crate's own public surface or MSRV — then **MINOR** |

Rules:

- **Pre-1.0 (`0.y.z`) is not an excuse.** While the version is `0.y.z`, treat `y` as MAJOR and
  `z` as MINOR/PATCH, and still document every break.
- **No silent breaking changes.** A breaking change lands with a `## [Unreleased]` entry under
  `### Removed` or `### Changed`, plus a migration note.
- **Deprecate before removing.** Mark the item `#[deprecated(since = "...", note = "use ... instead")]`
  in one release; remove it no earlier than the next MAJOR.
- **Raising the MSRV is at least a MINOR bump** and must be stated in the changelog.
- **Every released version is built from the version-control tree and gets a tag** (`vX.Y.Z`).
  Never ship a build that does not correspond to a tagged commit.
- Bump `version` in `Cargo.toml` and move the `## [Unreleased]` block to a dated
  `## [X.Y.Z] - YYYY-MM-DD` heading in the same commit as the tag.

---

## 3. Keep the changelog updated

`CHANGELOG.md` follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). **Every**
user-visible change updates it — in the same commit, not afterwards.

- Add the entry under `## [Unreleased]` in the correct category: `Added`, `Changed`,
  `Deprecated`, `Removed`, `Fixed`, `Security`. Use `Security` only for vulnerability fixes,
  and name the advisory (`RUSTSEC-...`, `CVE-...`) when there is one.
- Write for the person **consuming** the change, not for the reviewer: what changed and what
  they must do about it. Not `refactor handler`, but
  `HTTP handlers now return 422 instead of 400 for validation errors`.
- Internal-only refactors with no observable effect may be omitted; when in doubt, include them.
- The entry must make clear **what changed and where** (files, modules, endpoints), so a
  reviewer can reconstruct the change from the changelog alone.
- Never rewrite a released section. Corrections go in a new entry.

---

## 4. Tests: unit *and* behaviour, coverage above 85%

Two layers are required. A feature is not done with only one of them.

### 4.1 Unit / integration tests

- `#[test]` (and `#[tokio::test]`) functions in `src/` (`mod tests`) and `tests/*.rs`.
- Cover the happy path, every error branch, and the boundaries — empty input, maximum length,
  zero, overflow, unauthorized caller.
- Tests are deterministic: no wall-clock, no network, no shared global state, no ordering
  dependence between tests. Inject a `Clock`, a repository, an HTTP client.
- Every fixed bug gains a regression test that fails without the fix.

### 4.2 Behaviour (BDD) tests

- Gherkin `.feature` files under `tests/features/`, run by
  [cucumber-rs](https://crates.io/crates/cucumber), with step definitions in `tests/bdd.rs`
  (see the worked example earlier in this file).
- Every user-facing feature and every security control (authentication, authorization,
  lockout, input limits) gets at least one scenario, **including the negative case** — access
  denied, input rejected, lockout triggered.
- Keep steps thin: parse arguments, call one helper from `src/`, assert.

### 4.3 Coverage gates: 85% overall, 95% security-sensitive

Two floors. Both are gates, not targets, and `make coverage` enforces both.

| Scope | Floor |
|---|---|
| Every line of the project | **85%** |
| Every file listed in [`.security-sensitive`](.security-sensitive) | **95%** |

```bash
cargo install cargo-llvm-cov                       # once
make coverage                                      # both floors (scripts/coverage-gate.py)
cargo llvm-cov --all-features --workspace --html   # browse uncovered lines
```

- **Line coverage must stay above 85% overall.** A change that pushes it below the threshold is
  not mergeable; add the missing tests instead of lowering the gate.
- **Security-sensitive code must stay above 95%**, and every one of its error branches must be
  covered: the rejected input, the denied caller, the expired token, the triggered lockout, the
  failed audit write. A security control whose negative case is untested is not tested.
- A file is security-sensitive when it implements or enforces a control from section 5 or 6 —
  authentication, authorization, session or token handling, password hashing, crypto, input
  validation, output escaping, query construction, secret loading, lockout, rate limiting, or the
  audit trail. **Declare it in `.security-sensitive` in the same commit that creates it.** An
  undeclared security module is an unenforced 95%, which is the failure mode the manifest exists
  to prevent; reviewers check it against the diff.
- Coverage never goes **down** in a pull request, even while above a floor.
- Do not chase the number with assertion-free tests. An uncovered error branch means a missing
  test; a test that executes code without asserting on it is worse than no test.
- Any coverage exclusion needs a comment justifying why the code is untestable. An exclusion is
  never how a file reaches 95%.

---

## 5. Secure development practices

These are requirements, not suggestions. Reviewers reject changes that violate them.

The [`secure-development` skill](.claude/skills/secure-development/SKILL.md) is the working form
of this section and section 6: the same rules, with the Rust patterns that satisfy them and the
review checklist to run before calling a change done. Claude Code loads it automatically; other
assistants should be pointed at it. Keep the two in step — where they disagree, this file wins
and the disagreement is a bug to fix in the same pull request.

### 5.1 Authentication and authorization

- **One** authentication and authorization entry point for the whole application. Never
  re-implement a check inline in a handler, a command, or a UI callback.
- Authorize by **role/group**, never by hard-coded user identity, with granularity per
  application function.
- Require a second factor for sensitive operations: creating or changing credentials, changing
  a password, changing configuration, exporting data, restoring a backup, changing permissions.
- Apply progressive lockout on login — e.g. 3 failures -> 1 min, 5 -> 15 min, 7 -> 1 h — keyed
  primarily on client IP, enforced **before** the password is checked, server-side. Return an
  identical response for "unknown user" and "wrong password".

### 5.2 Data and secrets

- **No secret ever enters the repository** — not in code, not in committed config, not in
  tests, not in fixtures, not in the git history. Secrets come from the environment or a
  secret manager; `.env` is git-ignored and only `.env.example` (placeholders only) is
  committed.
- Passwords and anything else that never needs recovering are stored as a **one-way hash**
  with a modern KDF (Argon2id / scrypt). Never encrypt a password, never compare one in SQL.
- Data that must be reversible is decrypted **in server memory only**, for the shortest
  possible time, and zeroized after use (`zeroize`).
- Sensitive data travels **encrypted only**: TLS on every hop, including internal ones.
- Production data never reaches development or staging without passing through a masking step.
- Never log or persist a password (even a wrong one), token, key, session cookie, full
  document number, or unfiltered request body.

### 5.3 Code

- **Parameterized queries only.** Never interpolate input — or any part of a URL — into SQL.
  Where parameters cannot bind (table name, sort column, sort direction), use an allow-list
  with a safe default. Manual escaping is not an acceptable primary defence.
- **Escape on output.** No user-controlled HTML or JavaScript reaches a rendered page. Rely on
  the template engine's auto-escaping; sanitize with an allow-list where markup is genuinely
  allowed. Validate server-side — client-side validation is a convenience, not a control.
- **Bound every input**, including URLs, query strings, request bodies, and file uploads.
  Enforce the limit server-side and reject with a handled error.
- **Handle every error.** Log it with context and a correlation id; return a generic message
  carrying only that id. Never render a stack trace, SQL statement, file path, hostname, or
  component version. Never swallow an error silently — no bare `let _ =` on a `Result`, no
  `unwrap()`/`expect()`/`panic!` in request or command paths. Debug mode stays off outside
  local development.
- **No mutable global state fed by user input.** Configuration is loaded from a trusted source
  and treated as immutable at runtime. Inject dependencies instead of reaching for singletons.
- **Protect every service endpoint** with TLS plus an access key or token — read-only
  endpoints included — and restrict by source IP where the caller is predictable. Expose the
  minimum data needed.
- **Never build a diagnostic shortcut**: no arbitrary-SQL endpoint, no admin screen that runs
  free-form queries, no support backdoor, no flag that skips authentication outside
  production. Whoever adds one owns every misuse of it.
- Avoid heavy database work on unauthenticated surfaces; cache instead.

### 5.4 Dependencies

- Discontinued or unmaintained components are not allowed. Check the support horizon **before**
  adopting a dependency.
- Patch-level updates at least quarterly; key frameworks reviewed at least every six months.
- A **critical** vulnerability in a dependency outranks every feature request. Fix it first,
  and say so instead of continuing with the feature work.

---

## 6. Full audit coverage

"Audit" means two separate obligations. Both are mandatory.

### 6.1 Audit trail — who did what

An audit trail is not a log. Keep the two mechanisms separate.

| | Audit trail | Log |
|---|---|---|
| Purpose | Accountability | Diagnostics |
| Store | Separate instance from production data | Application log sink |
| Mutability | INSERT-only, enforced by the database | Rotated freely |

Requirements:

- Audit **every** security-relevant event. At minimum: sign-in (success **and** failure),
  sign-out, account creation, account change, credential creation or change, password change,
  permission change, configuration change, data export, backup restore, and every
  administrative action.
- Write the trail to a **different database instance** from production data.
- The application's database role holds `INSERT` **only** — no `UPDATE`, no `DELETE`. The
  immutability is enforced by database grants and constraints, never by application discipline.
- Optimize the table for cheap inserts: minimal indexes, no heavy triggers on the write path.
- Record at least: timestamp, actor, event, target, source IP, and a structured detail field.
  Never put a secret or sensitive value in the detail field.
- Asynchronous writes are fine; **silent loss is not**. A failure to audit raises an alert.
- Document the audit mechanism — events covered, schema, retention — under `docs/`.

### 6.2 Logging

- **One** logging library, used everywhere. No `println!`/`eprintln!` outside `main` startup.
- At least three levels: **Info** (routine), **Warn** (needs attention), **Error** (problems).
- Every error is logged. Authentication events and changes to important data are always logged.
- Configure the formatter once, centrally — do not assemble log strings at each call site. The
  house format is `[dd/mm/yyyy] hh:mm:ss ; event ; details`, for example:

  ```
  [29/07/2026] 14:32:05 ; login.failed ; user=jsilva ip=10.2.3.4 attempt=3 lockout=60s
  ```

  Structured (JSON) logging is acceptable provided it keeps the same three fields as keys.

### 6.3 Supply-chain and code audit

Run in CI on every pull request, and locally before a release:

```bash
make audit    # cargo audit + cargo deny check
make lint     # cargo clippy --all-targets -- -D warnings
```

- An advisory may only be added to a `deny.toml` ignore list with a written justification
  naming the upstream blocker and why the code path is unreachable. "Noisy" is not a
  justification.
- A **critical or high** finding blocks the release and blocks new feature work until fixed.
- Never commit `Cargo.lock` changes you have not reviewed.

---

## 7. Definition of done

A change is complete only when **all** of these hold:

- [ ] It maps to a roadmap item in [`docs/roadmap.md`](docs/roadmap.md), ticked in the same commit.
- [ ] [`CHANGELOG.md`](CHANGELOG.md) has an entry under `## [Unreleased]` in the right category.
- [ ] The version bump matches the semver rules in section 2 (or the change is unreleased).
- [ ] Unit/integration tests cover the happy path, the error branches, and the boundaries.
- [ ] At least one BDD scenario covers the behaviour, including its negative case.
- [ ] `make coverage` passes — 85% of lines overall, 95% on every security-sensitive file — and
      coverage did not drop.
- [ ] Every new file that implements or enforces a security control is listed in
      [`.security-sensitive`](.security-sensitive).
- [ ] `make fmt`, `make lint`, and `make test` pass.
- [ ] `make audit` is clean.
- [ ] Every security-relevant action the change introduces is audited (6.1) and logged (6.2).
- [ ] No secret, credential, or production data was added to the repository.
- [ ] Every new public item has a `///` doc comment; every new module has a `//!` comment.

---

## 8. Where the rules come from

- [`docs/roadmap.md`](docs/roadmap.md) — what to build, and in what order.
- [`docs/architecture.md`](docs/architecture.md) — how this project is laid out.
- [`.claude/skills/secure-development/SKILL.md`](.claude/skills/secure-development/SKILL.md) —
  sections 5 and 6 in working form, for you and for any AI assistant.
- [`.security-sensitive`](.security-sensitive) — which paths the 95% coverage floor applies to.
- [IronRoot AGENTS.md](https://github.com/ffquintella/IronRoot/blob/main/ai/AGENTS.md) — framework-wide agent rules.
- [IronRoot INSTRUCTIONS.md](https://github.com/ffquintella/IronRoot/blob/main/ai/INSTRUCTIONS.md) — naming, layout, and extension conventions.

If a rule here conflicts with your organisation's own security standard, the organisation's
standard wins — and the conflict belongs in a pull request against this file.
"##;

// --- docs renderers --------------------------------------------------------

fn docs_index_html(cfg: &ProjectConfig) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <title>{name} — Documentation</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0, minimum-scale=1.0" />
    <meta name="description" content="{name} — bootstrapped by ironroot" />
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/docsify@4/lib/themes/vue.css" />
    <style>
      :root {{
        --theme-color: #c75c2b;
      }}
      .sidebar > h1 {{ font-size: 1.3rem; }}
      .markdown-section pre > code {{ font-size: 0.85rem; }}
    </style>
  </head>
  <body>
    <div id="app">Loading…</div>
    <script>
      window.$docsify = {{
        name: '{name}',
        loadSidebar: true,
        loadNavbar: true,
        subMaxLevel: 3,
        auto2top: true,
        homepage: 'README.md',
        search: {{
          maxAge: 86400000,
          paths: 'auto',
          placeholder: 'Search…',
          noData: 'No results.',
          depth: 4,
        }},
        copyCode: {{
          buttonText: 'Copy',
          errorText: 'Error',
          successText: 'Copied',
        }},
        pagination: {{
          previousText: '← Previous',
          nextText: 'Next →',
          crossChapter: true,
        }},
      }};
    </script>
    <script src="https://cdn.jsdelivr.net/npm/docsify@4"></script>
    <script src="https://cdn.jsdelivr.net/npm/docsify@4/lib/plugins/search.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/docsify-copy-code@2"></script>
    <script src="https://cdn.jsdelivr.net/npm/docsify-pagination@2/dist/docsify-pagination.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/prismjs@1/components/prism-rust.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/prismjs@1/components/prism-toml.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/prismjs@1/components/prism-bash.min.js"></script>
  </body>
</html>
"#,
        name = cfg.name,
    )
}

fn docs_readme(cfg: &ProjectConfig) -> String {
    format!(
        r#"# {name}

> Bootstrapped by **ironroot**.

Welcome to the **{name}** documentation. The content is plain Markdown under
`docs/` and is rendered live by [docsify](https://docsify.js.org).

## Quick links

- [Getting started](getting-started.md) — build, test, and run
- [Architecture](architecture.md) — how this project is laid out
- [Roadmap](roadmap.md) — what is planned, and what is done

## Reading the docs locally

```bash
# Unix / macOS
./docs/serve-docs.sh

# Windows
docs\serve-docs.bat
```

Both scripts try `docsify-cli` first (`npm i -g docsify-cli`) and fall back
to Python's `http.server` if Node is not available. Then open
<http://localhost:3000>.

## Project status

- Project kind : {kind}
{gui_line}{frontend_line}- Database     : {db}

See [AGENTS.md](../AGENTS.md) for AI-assistant guidance.
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

fn docs_getting_started(cfg: &ProjectConfig) -> String {
    format!(
        r#"# Getting started

## Prerequisites

- A recent stable Rust toolchain (see `rust-toolchain.toml`).
- `make` (optional — every target maps to a plain `cargo` command).

## Build, test, run

```bash
make build    # cargo build --workspace
make test     # unit + integration + BDD
make bdd      # just the Gherkin scenarios
make run      # cargo run
```

## Configuration

Copy `.env.example` to `.env` and fill in real values:

```bash
cp .env.example .env
```

## Where things live

This page is generated for **{name}** ({kind}). Edit the Markdown files under
`docs/` to grow this site — docsify picks up changes on reload. Add new pages
to `docs/_sidebar.md` so they show up in the navigation.
"#,
        name = cfg.name,
        kind = cfg.kind.label(),
    )
}

fn docs_architecture(cfg: &ProjectConfig) -> String {
    let mut layout = String::new();
    match cfg.kind {
        ProjectKind::ClientTool => {
            layout.push_str("- `src/lib.rs` — the CLI's logic, with its unit tests\n");
            layout.push_str("- `src/logging.rs` — the single logging entry point\n");
            layout.push_str("- `src/main.rs` — collects the arguments, calls the library\n");
        }
        ProjectKind::WebApp => {
            layout
                .push_str("- `src/lib.rs` — the router and its handlers, with their unit tests\n");
            layout.push_str("- `src/logging.rs` — the single logging entry point\n");
            layout.push_str("- `src/main.rs` — reads the environment, binds, serves the router\n");
        }
        ProjectKind::ClientServer => {
            layout.push_str(
                "- `server/` — HTTP server (axum); logic in `src/lib.rs`, tests in `tests/`\n",
            );
            layout.push_str(
                "- `client/` — desktop GUI client; logic in `src/lib.rs`, tests in `tests/`\n",
            );
        }
    }
    if cfg.frontend.is_some() {
        layout.push_str("- `frontend/` — JS/TS frontend\n");
    }
    layout.push_str("- `docs/` — this documentation site (docsify)\n");
    if !matches!(cfg.kind, ProjectKind::ClientServer) {
        layout.push_str("- `tests/` — integration (`smoke.rs`) + BDD (`features/`, `bdd.rs`)\n");
    }

    format!(
        r#"# Architecture

**{name}** follows the IronRoot conventions: thin entrypoints, business logic
in small focused helper modules under `src/`, and behaviour covered by both
unit tests and BDD scenarios.

## Layout

{layout}
## Principles

1. **Keep entrypoints thin** — `main.rs` collects the input and calls the
   library. It is not a style preference: a binary target is not linked by
   `tests/*.rs` and is unreachable from a cucumber step, so code that lives in
   `main` cannot be covered by either of the two mandatory test layers while
   still counting against the 85% floor in [AGENTS.md](../AGENTS.md) §4.3.
2. **One concern per module.** Domain logic and I/O do not mix.
3. **Pure first, side-effects at the edges** — keep helpers testable. Handlers
   and UI callbacks move values; the helper they call makes the decision.

See [AGENTS.md](../AGENTS.md) for the full set of house rules.
"#,
        name = cfg.name,
    )
}

/// Seed roadmap. `AGENTS.md` makes this file the planning unit — every change
/// has to map to an item here — so the generated project ships with one
/// instead of asking the first contributor to invent the convention.
fn docs_roadmap(cfg: &ProjectConfig) -> String {
    format!(
        r#"# Roadmap

Planning for **{name}** ({kind}). Every change maps to an item on this page — see
[AGENTS.md](../AGENTS.md) §1. Add the item *before* the work, tick it in the same commit that
lands the work, and record the result in [CHANGELOG.md](../CHANGELOG.md).

## Phase 1 — Foundations

- [x] Project bootstrapped by `ironroot`
- [x] `make test` and `make coverage` green as generated — 85% of lines overall
- [ ] Replace the placeholder logic in `src/lib.rs` with the real thing
- [ ] First business-logic helper module of your own under `src/`, with unit tests
- [ ] First BDD scenario covering it (`tests/features/`)
- [ ] `make coverage` still green after it: 85% overall, 95% on security-sensitive paths
- [ ] `make audit` wired into CI

## Phase 2 — Security baseline

- [ ] Single authentication / authorization entry point
- [ ] Role-based authorization with per-function granularity
- [ ] Progressive login lockout (3 -> 1 min, 5 -> 15 min, 7 -> 1 h), keyed on IP
- [ ] Second factor on sensitive operations
- [ ] Central logging library configured with the house format
- [ ] Audit trail on a separate, INSERT-only instance
- [ ] Audit mechanism documented under `docs/`
- [ ] Every security module declared in `.security-sensitive` and covered above 95%

## Phase 3 — Features

- [ ] _Add your feature items here._

## Phase 4 — Release

- [ ] Version set per Semantic Versioning, `CHANGELOG.md` section dated
- [ ] Tag `vX.Y.Z` cut from the version-control tree
- [ ] Clean `make lint`, `make test`, `make coverage`, `make audit`
"#,
        name = cfg.name,
        kind = cfg.kind.label(),
    )
}

// --- docs blobs ------------------------------------------------------------

const DOCS_SIDEBAR: &str = r#"<!-- docs/_sidebar.md -->

- **Overview**
  - [Home](README.md)
  - [Getting started](getting-started.md)
  - [Architecture](architecture.md)
  - [Roadmap](roadmap.md)
"#;

const DOCS_NAVBAR: &str = r#"<!-- docs/_navbar.md -->

- [Home](/)
- [Getting started](getting-started.md)
- [Architecture](architecture.md)
- [Roadmap](roadmap.md)
"#;

const DOCS_SERVE_SH: &str = r#"#!/usr/bin/env bash
# Serve this project's docsify site locally.
#
# Tries `docsify-cli` first (best DX: live reload, sidebar autoreload).
# Falls back to `python3 -m http.server`, then `python -m http.server`.

set -euo pipefail

DOCS_DIR="$(cd "$(dirname "$0")" && pwd)"
PORT="${PORT:-3000}"

cd "$DOCS_DIR"

echo "Serving $DOCS_DIR on http://localhost:$PORT"
echo

if command -v docsify >/dev/null 2>&1; then
    echo "Using docsify-cli."
    exec docsify serve "$DOCS_DIR" --port "$PORT"
fi

if command -v npx >/dev/null 2>&1; then
    # Try docsify-cli via npx without forcing an install if it's not cached.
    if npx --no-install docsify-cli --version >/dev/null 2>&1; then
        echo "Using docsify-cli via npx."
        exec npx --no-install docsify-cli serve "$DOCS_DIR" --port "$PORT"
    fi
fi

if command -v python3 >/dev/null 2>&1; then
    echo "docsify-cli not found; falling back to python3 -m http.server."
    echo "Tip: install live-reload with: npm i -g docsify-cli"
    exec python3 -m http.server "$PORT" --bind 127.0.0.1
fi

if command -v python >/dev/null 2>&1; then
    echo "docsify-cli not found; falling back to python -m http.server."
    exec python -m http.server "$PORT" --bind 127.0.0.1
fi

echo "ERROR: no server available." >&2
echo "Install either docsify-cli (npm i -g docsify-cli) or Python 3." >&2
exit 1
"#;

const DOCS_SERVE_BAT: &str = r#"@echo off
REM Serve this project's docsify site locally (Windows).
REM Tries docsify-cli first, then npx, then Python's http.server.

setlocal
set "DOCS_DIR=%~dp0"
if "%PORT%"=="" set "PORT=3000"

cd /d "%DOCS_DIR%"

echo Serving %DOCS_DIR% on http://localhost:%PORT%
echo.

where docsify >nul 2>&1
if %ERRORLEVEL%==0 (
    echo Using docsify-cli.
    docsify serve "%DOCS_DIR%" --port %PORT%
    goto :eof
)

where npx >nul 2>&1
if %ERRORLEVEL%==0 (
    npx --no-install docsify-cli --version >nul 2>&1
    if %ERRORLEVEL%==0 (
        echo Using docsify-cli via npx.
        npx --no-install docsify-cli serve "%DOCS_DIR%" --port %PORT%
        goto :eof
    )
)

where python >nul 2>&1
if %ERRORLEVEL%==0 (
    echo docsify-cli not found; falling back to python -m http.server.
    echo Tip: install live-reload with: npm i -g docsify-cli
    python -m http.server %PORT% --bind 127.0.0.1
    goto :eof
)

where py >nul 2>&1
if %ERRORLEVEL%==0 (
    echo docsify-cli not found; falling back to py -m http.server.
    py -m http.server %PORT% --bind 127.0.0.1
    goto :eof
)

echo ERROR: no server available.
echo Install either docsify-cli (npm i -g docsify-cli) or Python 3.
exit /b 1
"#;

// --- secure-development blobs ---------------------------------------------

/// `.claude/skills/secure-development/SKILL.md`. A Claude Code skill carrying the secure
/// development rules from `AGENTS.md` §5–§6 in the form an assistant loads on its own: what
/// counts as untrusted input, parameterized queries, the single authentication entry point,
/// secret handling, error handling, the audit trail, dependency hygiene, the two coverage
/// floors, and the checklist to run before calling a change done. Kept byte-identical to the
/// copy in `templates/*/.claude/skills/` — change both or neither.
const SECURE_DEVELOPMENT_SKILL: &str = r##"---
name: secure-development
description: The binding secure-development rules for this project. Use when writing, reviewing, or designing any code that touches authentication, authorization, passwords, hashing, tokens, sessions, crypto, secrets or configuration, SQL and other queries, user input, request bodies, file uploads, HTTP handlers, CLI arguments, IPC commands, UI callbacks, error handling, logging, or the audit trail — and when adding a dependency, reviewing a diff, writing tests for security-sensitive code, or checking whether a change is done. Enforces the 85% overall / 95% security-sensitive line-coverage floors.
---

# Secure development

The rules below are the project's own, not general advice. They restate
[`AGENTS.md`](../../../AGENTS.md) §4–§7 in the form you need while writing code. Where this file
and `AGENTS.md` disagree, `AGENTS.md` wins and the disagreement is a bug — fix it in the same
pull request.

**A change that violates one of these is rejected, not negotiated.** If a rule genuinely cannot
be met, say so explicitly in the pull request with the reason — never skip it silently and never
weaken the gate instead of the code.

## Non-negotiables at a glance

| # | Rule |
|---|---|
| 1 | One authentication/authorization entry point. Never re-check inline. |
| 2 | Authorize by role/group, per application function. Never by hard-coded identity. |
| 3 | No secret in the repository — code, config, tests, fixtures, or git history. |
| 4 | Passwords are one-way hashes (Argon2id/scrypt). Never encrypted, never compared in SQL. |
| 5 | Parameterized queries only. Allow-lists where a parameter cannot bind. |
| 6 | Bound every input, server-side. Escape every output. |
| 7 | Handle every error. No `unwrap`/`expect`/`panic!` on a request, command, or callback path. |
| 8 | Every security-relevant action is audited (INSERT-only, separate store) and logged. |
| 9 | `unsafe` is forbidden unless unavoidable, isolated, `// SAFETY:`-justified, and tested. |
| 10 | 85% line coverage overall; **95% on every security-sensitive file**. |

## 1. Treat these as untrusted input — always

HTTP request bodies, query strings, path segments, headers, cookies, CLI arguments, environment
variables, file paths, stdin, file uploads, IPC/command payloads, UI form fields, and anything
read back from another service. Untrusted means: bound its size, validate its shape server-side,
reject what you do not recognise, and never interpolate it into a query, a path, a shell command,
or a rendered page.

```rust
// Bound the input before you parse it, not after.
const MAX_NAME: usize = 64;

pub fn parse_name(raw: &str) -> Result<Name, ValidationError> {
    if raw.is_empty() || raw.chars().count() > MAX_NAME {
        return Err(ValidationError::Length { max: MAX_NAME });
    }
    if !raw.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(ValidationError::Charset);
    }
    Ok(Name(raw.to_owned()))
}
```

Client-side validation is a convenience, never a control. Re-validate on the server even when
the UI already did.

## 2. Queries

Bind every value. Never build SQL by concatenation or `format!`, not even "just for the table
name in an internal admin tool".

```rust
// Correct — the value is bound, never interpolated.
let user = sqlx::query_as!(
    User,
    "SELECT id, email, role FROM users WHERE email = $1",
    email.as_str(),
)
.fetch_optional(&pool)
.await?;
```

Where a parameter cannot bind — table name, column name, sort direction — map the input through
an allow-list with a safe default. Manual escaping is not an acceptable primary defence.

```rust
fn sort_column(requested: &str) -> &'static str {
    match requested {
        "email" => "email",
        "created_at" => "created_at",
        _ => "id", // safe default; unknown input is not an error path worth leaking
    }
}
```

## 3. Authentication and authorization

- **One** entry point for the whole application. A handler, command, or UI callback asks it; it
  never re-implements the check.
- Authorize by **role/group**, with granularity per application function. Never by a hard-coded
  user id, email, or username.
- Require a **second factor** for sensitive operations: creating or changing credentials,
  changing a password, changing configuration or permissions, exporting data, restoring a backup.
- Apply **progressive lockout** on sign-in — e.g. 3 failures → 1 min, 5 → 15 min, 7 → 1 h — keyed
  primarily on client IP, enforced **before** the password is checked, server-side.
- Return an **identical** response for "unknown user" and "wrong password" — same body, same
  status, and no timing tell (verify against a dummy hash when the user does not exist).

```rust
// Same work, same answer, whether or not the account exists.
let stored = repo.password_hash(&email).await?;
let hash = stored.as_deref().unwrap_or(DUMMY_ARGON2_HASH);
let ok = verify_password(candidate, hash) && stored.is_some();
if !ok {
    audit.record(Event::LoginFailed { email: &email, ip }).await?;
    return Err(AuthError::InvalidCredentials); // one variant for both cases
}
```

## 4. Secrets and sensitive data

- Secrets come from the environment or a secret manager. `.env` is git-ignored; only
  `.env.example` with placeholder values is committed. **A secret that reaches a commit is
  burned** — rotate it, do not just delete the line.
- Passwords and anything that never needs recovering: one-way hash with a modern KDF (Argon2id,
  scrypt). Never encrypt a password. Never compare one in SQL.
- Data that must be reversible is decrypted in server memory only, for the shortest possible
  time, and zeroized after use (`zeroize`).
- TLS on every hop, internal ones included. No plaintext endpoint anywhere.
- Production data never reaches development or staging without masking first.
- **Never log or persist**: a password (even a wrong one), token, key, session cookie, full
  document/ID number, or an unfiltered request body. Derive a `Debug` impl by hand for any type
  holding one, or wrap it so the value cannot print.

```rust
pub struct Secret(String);

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(***)") // a `#[derive(Debug)]` here would leak on every `?` log
    }
}
```

## 5. Errors

- Handle every error. Log it with context and a correlation id; return a generic message that
  carries only that id.
- Never render a stack trace, SQL statement, file path, hostname, or component version to a
  caller. Debug mode stays off outside local development.
- No silent swallowing: no bare `let _ =` on a `Result`, no `unwrap()`, `expect()`, or `panic!`
  on a request, command, or callback path. An unhandled panic is a bug, not an error path.
- Define a domain error per module with `thiserror` and map it at the edge.

```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("account locked")]
    Locked { retry_after: std::time::Duration },
}

// At the edge: log the detail, return the id.
tracing::error!(correlation_id = %id, error = ?err, "request failed");
(StatusCode::INTERNAL_SERVER_ERROR, format!("request failed (ref {id})"))
```

## 6. Audit trail and logging

They are two mechanisms and stay separate. The audit trail is for accountability; the log is for
diagnostics.

Audit **every** security-relevant event: sign-in success **and** failure, sign-out, account
creation or change, credential creation or change, password change, permission change,
configuration change, data export, backup restore, every administrative action. Record at least
timestamp, actor, event, target, source IP, and a structured detail field — with no secret or
sensitive value in the detail.

The trail lives on a **separate database instance** from production data, and the application's
role holds `INSERT` **only**; immutability is enforced by database grants, never by application
discipline. Asynchronous writes are fine; **silent loss is not** — a failure to audit raises an
alert.

Logging: one library, used everywhere, configured centrally. No `println!`/`eprintln!` outside
`main` startup. At least Info/Warn/Error. House format:

```
[dd/mm/yyyy] hh:mm:ss ; event ; details
[29/07/2026] 14:32:05 ; login.failed ; user=jsilva ip=10.2.3.4 attempt=3 lockout=60s
```

Structured JSON logging is fine provided it keeps the same three fields as keys.

## 7. Nothing that bypasses the rules

No arbitrary-SQL endpoint. No admin screen that runs free-form queries. No support backdoor. No
flag that skips authentication outside production. No mutable global state fed by user input —
configuration is loaded from a trusted source and immutable at runtime; inject dependencies
instead of reaching for singletons. Whoever adds a shortcut owns every misuse of it.

Protect every service endpoint — read-only ones included — with TLS plus an access key or token,
and restrict by source IP where the caller is predictable. Expose the minimum data needed. Avoid
heavy database work on unauthenticated surfaces; cache instead.

## 8. Dependencies

Check the support horizon **before** adopting a dependency; discontinued or unmaintained
components are not allowed. Patch-level updates at least quarterly. A **critical** vulnerability
in a dependency outranks every feature request — fix it first and say so.

```bash
cargo audit          # RUSTSEC advisories
cargo deny check     # advisories, bans, licenses, sources
```

An advisory may only be ignored in `deny.toml` with a written justification naming the upstream
blocker and why the code path is unreachable. "Noisy" is not a justification. Never commit a
`Cargo.lock` change you have not reviewed.

## 9. Coverage: 85% overall, 95% security-sensitive

Both floors are gates, not targets.

```bash
# Overall floor — 85% line coverage.
cargo llvm-cov --all-features --workspace --fail-under-lines 85

# Both floors at once: 85% overall plus 95% on every path in `.security-sensitive`.
./scripts/coverage-gate.py

# Browse what is uncovered.
cargo llvm-cov --all-features --workspace --html
```

- **Any file that implements or enforces one of the rules above is security-sensitive** —
  authentication, authorization, session and token handling, password hashing, crypto, input
  validation, output escaping, query construction, secret loading, the audit trail, lockout,
  rate limiting.
- Declare it in [`.security-sensitive`](../../../.security-sensitive) **in the same commit that
  creates it**. An undeclared security module is an unenforced 95%, which is the whole failure
  mode this gate exists to prevent. Reviewers check the manifest against the diff.
- For those files, 95% of lines is the floor and **every error branch is covered**: the rejected
  input, the denied caller, the expired token, the triggered lockout, the failed audit write. A
  security control with an untested negative case is not tested.
- Coverage never goes **down**, even while above the floor. Add the missing test instead of
  lowering the gate.
- Do not chase the number. A test that executes code without asserting on it is worse than no
  test; an uncovered error branch means a missing test, not an excludable line. Any coverage
  exclusion carries a comment saying why the code is untestable.

Both test layers are mandatory (`AGENTS.md` §4): unit/integration tests **and** at least one
Gherkin scenario per feature and per security control — **including the negative case**: access
denied, input rejected, lockout triggered.

```gherkin
Scenario: A caller without the auditor role cannot export data
  Given a signed-in user with the "viewer" role
  When they request a data export
  Then the request is denied with a generic error
  And the denial is recorded in the audit trail
```

## 10. Before you call it done

- [ ] Every new input is bounded and validated server-side; every output escaped.
- [ ] Every query binds its values; every non-bindable part goes through an allow-list.
- [ ] Authentication and authorization go through the single entry point, by role.
- [ ] No secret, credential, or production data entered the repository.
- [ ] Every error is handled and logged with a correlation id; no `unwrap`/`expect`/`panic!` on a
      request, command, or callback path; no bare `let _ =` on a `Result`.
- [ ] Every security-relevant action is audited and logged.
- [ ] New security-sensitive files are listed in `.security-sensitive`.
- [ ] `./scripts/coverage-gate.py` passes — 85% overall, 95% on security-sensitive files — and
      coverage did not drop.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
      `cargo audit`, and `cargo deny check` are clean.
- [ ] `CHANGELOG.md` has the entry (`Security` category if it fixes a vulnerability, with the
      advisory id), and the roadmap item is ticked.

If your organisation's own security standard conflicts with a rule here, the organisation's
standard wins — and the conflict belongs in a pull request against `AGENTS.md`.
"##;

/// `.security-sensitive`. The paths the 95% coverage floor applies to, seeded with the
/// conventional module names so the convention is discoverable before the first security
/// control lands.
const SECURITY_SENSITIVE: &str = r##"# Security-sensitive paths — the 95% line-coverage floor applies to every file listed here.
# See AGENTS.md §4.3 and .claude/skills/secure-development/SKILL.md §9.
#
# One `fnmatch` pattern per line, relative to this directory, `#` starts a comment.
# `*` crosses directory separators, so `*src/auth*` covers `src/auth.rs`, `src/auth/token.rs`,
# and `server/src/auth.rs` in a workspace.
# Checked by `./scripts/coverage-gate.py`.
#
# A file belongs here when it implements or enforces a security control:
# authentication, authorization, session or token handling, password hashing, crypto,
# input validation, output escaping, query construction, secret loading, lockout, rate
# limiting, or the audit trail.
#
# Add the path in the same commit that creates the file. An undeclared security module is
# an unenforced 95% — which is the whole failure mode this manifest exists to prevent.
# Reviewers check this file against the diff.
#
# The patterns below are the conventional names, active and matching nothing yet. Keep them,
# add yours, and delete a line only when the concept genuinely does not exist here.

*src/auth*
*src/authz*
*src/security*
*src/crypto*
*src/session*
*src/token*
*src/password*
*src/permission*
*src/audit*
*src/validation*
"##;

/// `scripts/coverage-gate.py`. Enforces both floors from `AGENTS.md` §4.3 — 85% of lines
/// overall, 95% on every path in `.security-sensitive` — because
/// `cargo llvm-cov --fail-under-lines` can only express one project-wide number. Python runs
/// the gate only; nothing in the generated build depends on it.
const COVERAGE_GATE_PY: &str = r##"#!/usr/bin/env python3
"""Two-threshold line-coverage gate — see AGENTS.md §4.3.

    85%  line coverage over the whole project
    95%  line coverage on every file declared in `.security-sensitive`

`cargo llvm-cov --fail-under-lines` enforces a single project-wide number, which is why the
second floor needs a few lines of glue: this script asks cargo-llvm-cov for its JSON export and
applies both thresholds to it. Python only runs the gate — nothing in the build depends on it.

Usage:
    ./scripts/coverage-gate.py                 # collect coverage, then check both floors
    ./scripts/coverage-gate.py --json cov.json # re-check an existing llvm-cov JSON export
    ./scripts/coverage-gate.py --min-all 90    # raise a floor (lowering one is not a fix)

Exit status: 0 both floors met, 1 a floor was missed, 2 the gate could not run.
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import NoReturn

MIN_ALL = 85.0
MIN_SECURITY = 95.0
MANIFEST = ".security-sensitive"


def project_root() -> Path:
    """The directory holding `Cargo.toml` — this script lives in `<root>/scripts/`."""
    return Path(__file__).resolve().parent.parent


def load_patterns(root: Path) -> list[str]:
    """Read the security-sensitive path patterns, skipping blanks and `#` comments."""
    manifest = root / MANIFEST
    if not manifest.is_file():
        return []
    patterns = []
    for line in manifest.read_text(encoding="utf-8").splitlines():
        line = line.split("#", 1)[0].strip()
        if line:
            patterns.append(line)
    return patterns


def collect(root: Path, destination: Path) -> None:
    """Run the test suite under instrumentation and write the JSON export."""
    if shutil.which("cargo-llvm-cov") is None:
        fail(
            "cargo-llvm-cov is not installed.\n"
            "    cargo install cargo-llvm-cov\n"
            "Or pass an existing export with --json."
        )
    subprocess.run(
        [
            "cargo",
            "llvm-cov",
            "--all-features",
            "--workspace",
            "--summary-only",
            "--json",
            "--output-path",
            str(destination),
        ],
        cwd=root,
        check=True,
    )


def read_report(path: Path, root: Path) -> tuple[float, list[tuple[str, int, int, float]]]:
    """Return the project-wide line percentage and a per-file `(path, covered, count, pct)` list.

    Paths are made relative to the project root; anything outside it (a dependency compiled from
    a local checkout, say) is not this project's code and is dropped.
    """
    export = json.loads(path.read_text(encoding="utf-8"))
    data = export["data"][0]
    files = []
    for entry in data["files"]:
        try:
            relative = Path(entry["filename"]).resolve().relative_to(root)
        except ValueError:
            continue
        lines = entry["summary"]["lines"]
        files.append(
            (relative.as_posix(), lines["covered"], lines["count"], float(lines["percent"]))
        )
    return float(data["totals"]["lines"]["percent"]), sorted(files)


def matches(relative: str, patterns: list[str]) -> bool:
    """`fnmatch` semantics, so `*` crosses directory separators: `src/auth*` covers the tree."""
    return any(fnmatch.fnmatch(relative, pattern) for pattern in patterns)


def fail(message: str) -> NoReturn:
    print(f"coverage-gate: {message}", file=sys.stderr)
    raise SystemExit(2)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--json",
        type=Path,
        metavar="PATH",
        help="check an existing llvm-cov JSON export instead of collecting coverage",
    )
    parser.add_argument("--min-all", type=float, default=MIN_ALL, metavar="PCT")
    parser.add_argument("--min-security", type=float, default=MIN_SECURITY, metavar="PCT")
    args = parser.parse_args()

    root = project_root()
    patterns = load_patterns(root)

    with tempfile.TemporaryDirectory() as scratch:
        report = args.json
        if report is None:
            report = Path(scratch) / "coverage.json"
            try:
                collect(root, report)
            except subprocess.CalledProcessError as err:
                fail(f"cargo llvm-cov exited with {err.returncode}; fix the tests first.")
        elif not report.is_file():
            fail(f"{report} does not exist.")
        try:
            total, files = read_report(report, root)
        except (KeyError, IndexError, ValueError) as err:
            fail(f"{report} is not an llvm-cov JSON export ({err}).")

    sensitive = []

    print()
    print(f"{'file':<52} {'lines':>13} {'covered':>9}   gate")
    print("-" * 88)
    for relative, covered, count, pct in files:
        gate = ""
        if matches(relative, patterns):
            sensitive.append((relative, pct))
            gate = f"security ≥ {args.min_security:.0f}%"
            if pct < args.min_security:
                gate += " FAIL"
        print(f"{relative:<52} {f'{covered}/{count}':>13} {pct:>8.2f}%   {gate}")
    print("-" * 88)
    print(f"{'TOTAL':<52} {'':>13} {total:>8.2f}%   all ≥ {args.min_all:.0f}%")
    print()

    failures = []
    if total < args.min_all:
        failures.append(
            f"overall line coverage {total:.2f}% is below the {args.min_all:.0f}% floor"
        )
    for relative, pct in sensitive:
        if pct < args.min_security:
            failures.append(
                f"{relative} is security-sensitive and covers {pct:.2f}% of lines, "
                f"below the {args.min_security:.0f}% floor"
            )

    if not patterns:
        print(
            f"note: no patterns in {MANIFEST}, so the {args.min_security:.0f}% floor matched "
            "nothing.\n"
            "      Declare every file that implements or enforces a security control there,\n"
            "      in the same commit that creates it — see AGENTS.md §4.3."
        )
    elif not sensitive:
        print(
            f"note: the {len(patterns)} pattern(s) in {MANIFEST} matched no covered file yet.\n"
            f"      That is expected until the first security control lands; keep {MANIFEST}\n"
            "      in step with the tree so the floor applies the moment one does."
        )

    if failures:
        print()
        for message in failures:
            print(f"FAIL: {message}")
        print()
        print(
            "Add the missing tests — the error branches first. Lowering a floor is not a fix.\n"
            "Browse the gaps with: cargo llvm-cov --all-features --workspace --html"
        )
        return 1

    print(
        f"OK: {total:.2f}% overall (floor {args.min_all:.0f}%), "
        f"{len(sensitive)} security-sensitive file(s) at or above {args.min_security:.0f}%."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
"##;

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

/// Like [`write`], but marks the file executable on Unix (for shell scripts).
fn write_exec(dir: &Path, name: &str, contents: &str) -> io::Result<()> {
    write(dir, name, contents)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join(name);
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms)?;
    }
    Ok(())
}
