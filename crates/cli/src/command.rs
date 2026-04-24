//! Individual command abstraction.

/// Represents a single CLI command (sub-command).
///
/// Implement this trait to define a runnable command in an IronRoot CLI
/// application.
///
/// # TODO
///
/// - Add argument definitions (`args` method).
/// - Add asynchronous variant (`async fn execute`).
pub trait Command {
    /// Returns the name of the command as it appears on the command line.
    fn name(&self) -> &str;

    /// Returns a short description shown in help output.
    fn description(&self) -> &str;

    /// Executes the command with the provided raw arguments.
    ///
    /// Returns `Ok(())` on success or an error message on failure.
    fn execute(&self, args: &[String]) -> Result<(), String>;
}
