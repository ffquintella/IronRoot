//! # ironroot
//!
//! Interactive project bootstrapper for the IronRoot framework.
//!
//! Run with:
//!
//! ```bash
//! cargo run -p ironroot
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

    let needs_frontend = matches!(kind, ProjectKind::WebApp) || matches!(gui, Some(Gui::Tauri));
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

    let target_dir = prompts::ask_string("Target directory", Some(&format!("./{}", name)))?;
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
        let blocking = blocking_entries(&target)
            .map_err(|e| format!("cannot read {}: {e}", target.display()))?;
        if !blocking.is_empty() {
            return Err(format!(
                "target directory {} exists and is not empty (found {})",
                target.display(),
                describe_entries(&blocking)
            ));
        }
    }

    generator::generate(&cfg, &target).map_err(|e| format!("failed to generate project: {e}"))?;

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

/// Directory entries that do not count as "the target directory is occupied".
///
/// Running `git init` (or cloning an empty repository) and then bootstrapping
/// into that directory is a normal flow, but it leaves `.git/` behind, which
/// used to make the target look occupied. VCS metadata and OS scratch files are
/// not project files, so they are ignored. `.gitignore` deliberately is *not*
/// on this list — the generator writes one, so an existing one is a real
/// conflict.
const IGNORED_ENTRIES: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    ".jj",
    ".DS_Store",
    "Thumbs.db",
    ".keep",
    ".gitkeep",
];

/// Names in `dir` that block generation, sorted; empty means "effectively empty".
fn blocking_entries(dir: &Path) -> io::Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in dir.read_dir()? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        if !IGNORED_ENTRIES.contains(&name.as_str()) {
            names.push(name);
        }
    }
    names.sort();
    Ok(names)
}

/// Render blocking entry names for an error message, capped so a directory with
/// hundreds of files does not print hundreds of names.
fn describe_entries(names: &[String]) -> String {
    const MAX_SHOWN: usize = 5;
    let shown = names
        .iter()
        .take(MAX_SHOWN)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    match names.len().checked_sub(MAX_SHOWN) {
        Some(rest) if rest > 0 => format!("{shown} and {rest} more"),
        _ => shown,
    }
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("project name cannot be empty".into());
    }
    let ok = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !ok {
        return Err("project name must contain only ASCII letters, digits, '-' or '_'".into());
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn fresh_git_repo_counts_as_empty() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir(dir.path().join(".git")).expect("mkdir .git");
        fs::write(
            dir.path().join(".git").join("HEAD"),
            "ref: refs/heads/main\n",
        )
        .expect("write");
        fs::write(dir.path().join(".DS_Store"), "").expect("write");

        assert!(blocking_entries(dir.path()).expect("read").is_empty());
    }

    #[test]
    fn real_files_still_block() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir(dir.path().join(".git")).expect("mkdir .git");
        fs::write(dir.path().join("Cargo.toml"), "").expect("write");
        fs::write(dir.path().join(".gitignore"), "").expect("write");

        assert_eq!(
            blocking_entries(dir.path()).expect("read"),
            vec![".gitignore".to_owned(), "Cargo.toml".to_owned()]
        );
    }

    #[test]
    fn empty_dir_has_no_blocking_entries() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(blocking_entries(dir.path()).expect("read").is_empty());
    }

    #[test]
    fn describe_entries_caps_the_list() {
        let names: Vec<String> = (0..8).map(|i| format!("f{i}")).collect();
        assert_eq!(describe_entries(&names), "f0, f1, f2, f3, f4 and 3 more");
        assert_eq!(describe_entries(&names[..2]), "f0, f1");
    }
}
