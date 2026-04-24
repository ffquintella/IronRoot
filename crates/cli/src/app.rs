//! Top-level CLI application container.

/// The root container for an IronRoot CLI application.
///
/// `CliApp` holds registered [`crate::Command`] instances and is responsible
/// for parsing arguments and dispatching to the correct handler.
///
/// # TODO
///
/// - Add `register` method for adding commands.
/// - Add `run` method that parses `std::env::args` and dispatches.
pub struct CliApp {
    name: String,
    version: String,
}

impl CliApp {
    /// Creates a new CLI application with the given `name` and `version`.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        CliApp {
            name: name.into(),
            version: version.into(),
        }
    }

    /// Returns the application name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the application version string.
    pub fn version(&self) -> &str {
        &self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_app_stores_name_and_version() {
        let app = CliApp::new("my-tool", "0.1.0");
        assert_eq!(app.name(), "my-tool");
        assert_eq!(app.version(), "0.1.0");
    }
}
