//! Router abstraction — placeholder for future HTTP routing support.

/// A placeholder for the IronRoot web router.
///
/// # TODO
///
/// Implement route registration (`get`, `post`, `put`, `delete`) and
/// request dispatch in Phase 3 of the roadmap.
pub struct Router {
    // TODO: store registered routes
}

impl Router {
    /// Creates a new, empty router.
    pub fn new() -> Self {
        Router {}
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
