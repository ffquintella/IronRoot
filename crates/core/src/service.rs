//! Service trait — the foundation for all business-logic components in IronRoot.

/// Represents a component that encapsulates a unit of business logic.
///
/// Services are the primary vehicle for use-case implementation in IronRoot.
/// They are typically stateless (or hold only configuration/repository
/// references) and operate on [`crate::Entity`] instances.
///
/// # Implementing a service
///
/// ```rust
/// use ironroot_core::Service;
///
/// struct GreetingService {
///     prefix: String,
/// }
///
/// impl Service for GreetingService {
///     fn name(&self) -> &str {
///         "GreetingService"
///     }
/// }
///
/// impl GreetingService {
///     fn greet(&self, name: &str) -> String {
///         format!("{}, {}!", self.prefix, name)
///     }
/// }
///
/// let svc = GreetingService { prefix: "Hello".to_string() };
/// assert_eq!(svc.greet("Alice"), "Hello, Alice!");
/// ```
pub trait Service {
    /// Returns the human-readable name of this service, used for logging and
    /// diagnostics.
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyService;

    impl Service for DummyService {
        fn name(&self) -> &str {
            "DummyService"
        }
    }

    #[test]
    fn service_returns_name() {
        let svc = DummyService;
        assert_eq!(svc.name(), "DummyService");
    }
}
