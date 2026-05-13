//! # ironroot-new
//!
//! Interactive project bootstrapper for the IronRoot framework.
//!
//! Run with:
//!
//! ```bash
//! cargo run -p ironroot-new
//! ```
//!
//! The tool asks a handful of questions (project kind, GUI toolkit, database,
//! frontend framework) and then writes out a ready-to-build project skeleton
//! containing `Cargo.toml`, `Makefile`, basic tests, AI-agent instructions,
//! and a starter `src/` tree.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

mod config;
mod generator;
mod prompts;

use config::{Database, Frontend, Gui, ProjectConfig, ProjectKind};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    println!();
    println!("IronRoot — new project bootstrapper");
    println!("===================================");
    println!();

    let name = prompts::ask_string("Project name", Some("my-ironroot-app"))?;
    validate_name(&name)?;

    let kind = prompts::ask_choice(
        "What kind of project?",
        &[
            ("client-tool", "Command-line tool (CLI)"),
            ("webapp", "Web application (server-rendered or API + SPA)"),
            ("client-server", "Client / server (desktop GUI + backend)"),
        ],
        0,
    )?;
    let kind = match kind.as_str() {
        "client-tool" => ProjectKind::ClientTool,
        "webapp" => ProjectKind::WebApp,
        "client-server" => ProjectKind::ClientServer,
        _ => unreachable!(),
    };

    let gui = if matches!(kind, ProjectKind::ClientServer) {
        let g = prompts::ask_choice(
            "Which GUI toolkit for the client?",
            &[
                ("tauri", "Tauri (web tech inside a native window)"),
                ("egui", "egui (pure-Rust immediate-mode GUI)"),
            ],
            0,
        )?;
        Some(match g.as_str() {
            "tauri" => Gui::Tauri,
            "egui" => Gui::Egui,
            _ => unreachable!(),
        })
    } else {
        None
    };

    let needs_frontend =
        matches!(kind, ProjectKind::WebApp) || matches!(gui, Some(Gui::Tauri));
    let frontend = if needs_frontend {
        let f = prompts::ask_choice(
            "Which frontend framework?",
            &[
                ("react", "React (with Vite + TypeScript)"),
                ("angular", "Angular"),
            ],
            0,
        )?;
        Some(match f.as_str() {
            "react" => Frontend::React,
            "angular" => Frontend::Angular,
            _ => unreachable!(),
        })
    } else {
        None
    };

    let db = prompts::ask_choice(
        "Which database?",
        &[
            ("none", "None"),
            ("sqlite", "SQLite (via sqlx)"),
            ("postgres", "PostgreSQL (via sqlx)"),
            ("mysql", "MySQL / MariaDB (via sqlx)"),
        ],
        0,
    )?;
    let database = match db.as_str() {
        "none" => Database::None,
        "sqlite" => Database::Sqlite,
        "postgres" => Database::Postgres,
        "mysql" => Database::MySql,
        _ => unreachable!(),
    };

    let target_dir = prompts::ask_string(
        "Target directory",
        Some(&format!("./{}", name)),
    )?;
    let target = PathBuf::from(&target_dir);

    let cfg = ProjectConfig {
        name: name.clone(),
        kind,
        gui,
        frontend,
        database,
    };

    println!();
    println!("About to create project:");
    println!("  name      : {}", cfg.name);
    println!("  kind      : {}", cfg.kind.label());
    if let Some(g) = cfg.gui {
        println!("  gui       : {}", g.label());
    }
    if let Some(f) = cfg.frontend {
        println!("  frontend  : {}", f.label());
    }
    println!("  database  : {}", cfg.database.label());
    println!("  path      : {}", target.display());
    println!();

    if !prompts::ask_yes_no("Proceed?", true)? {
        println!("Aborted.");
        return Ok(());
    }

    if target.exists() {
        let empty = target
            .read_dir()
            .map_err(|e| format!("cannot read {}: {e}", target.display()))?
            .next()
            .is_none();
        if !empty {
            return Err(format!(
                "target directory {} exists and is not empty",
                target.display()
            ));
        }
    }

    generator::generate(&cfg, &target)
        .map_err(|e| format!("failed to generate project: {e}"))?;

    println!();
    println!("Project created at {}", target.display());
    println!();
    println!("Next steps:");
    println!("  cd {}", target.display());
    println!("  make build");
    println!("  make test");
    if let Some(frontend) = cfg.frontend {
        println!();
        println!(
            "Frontend skeleton ({}) is in `frontend/`. See its README for setup.",
            frontend.label()
        );
    }

    Ok(())
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("project name cannot be empty".into());
    }
    let ok = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !ok {
        return Err(
            "project name must contain only ASCII letters, digits, '-' or '_'".into(),
        );
    }
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        return Err("project name cannot start with a digit".into());
    }
    Ok(())
}

#[allow(dead_code)]
fn flush() {
    let _ = io::stdout().flush();
}

#[allow(dead_code)]
fn ensure_dir(p: &Path) -> io::Result<()> {
    std::fs::create_dir_all(p)
}
