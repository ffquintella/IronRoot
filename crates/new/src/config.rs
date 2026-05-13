//! Configuration captured from the interactive prompts.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    ClientTool,
    WebApp,
    ClientServer,
}

impl ProjectKind {
    pub fn label(self) -> &'static str {
        match self {
            ProjectKind::ClientTool => "CLI tool",
            ProjectKind::WebApp => "web application",
            ProjectKind::ClientServer => "client / server",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gui {
    Tauri,
    Egui,
}

impl Gui {
    pub fn label(self) -> &'static str {
        match self {
            Gui::Tauri => "Tauri",
            Gui::Egui => "egui",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frontend {
    React,
    Angular,
}

impl Frontend {
    pub fn label(self) -> &'static str {
        match self {
            Frontend::React => "React",
            Frontend::Angular => "Angular",
        }
    }

    pub fn dir_name(self) -> &'static str {
        match self {
            Frontend::React => "frontend",
            Frontend::Angular => "frontend",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Database {
    None,
    Sqlite,
    Postgres,
    MySql,
}

impl Database {
    pub fn label(self) -> &'static str {
        match self {
            Database::None => "none",
            Database::Sqlite => "SQLite",
            Database::Postgres => "PostgreSQL",
            Database::MySql => "MySQL",
        }
    }

    /// sqlx feature flag corresponding to the chosen database.
    pub fn sqlx_feature(self) -> Option<&'static str> {
        match self {
            Database::None => None,
            Database::Sqlite => Some("sqlite"),
            Database::Postgres => Some("postgres"),
            Database::MySql => Some("mysql"),
        }
    }

    /// Default DATABASE_URL for the generated `.env.example`.
    pub fn default_url(self) -> Option<&'static str> {
        match self {
            Database::None => None,
            Database::Sqlite => Some("sqlite://./data.db"),
            Database::Postgres => {
                Some("postgres://postgres:postgres@localhost:5432/app")
            }
            Database::MySql => Some("mysql://root:root@localhost:3306/app"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub kind: ProjectKind,
    pub gui: Option<Gui>,
    pub frontend: Option<Frontend>,
    pub database: Database,
}
