//! Desktop application container placeholder.

/// Placeholder for the IronRoot desktop application container.
///
/// In Phase 3, this struct will wrap the chosen UI framework (egui or Tauri)
/// and expose a unified API for window management, event handling, and
/// application lifecycle.
///
/// # TODO
///
/// - Choose UI backend (egui / Tauri) behind feature flags.
/// - Implement `run` method that starts the event loop.
pub struct GuiApp {
    title: String,
}

impl GuiApp {
    /// Creates a new desktop application with the given window `title`.
    pub fn new(title: impl Into<String>) -> Self {
        GuiApp { title: title.into() }
    }

    /// Returns the window title.
    pub fn title(&self) -> &str {
        &self.title
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gui_app_stores_title() {
        let app = GuiApp::new("My Desktop App");
        assert_eq!(app.title(), "My Desktop App");
    }
}
