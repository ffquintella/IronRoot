//! Behaviour tests for `ironroot-cli-app` — AGENTS.md §4.2.
//!
//! Gherkin sources live in `tests/features/`; this file only binds their steps
//! to helpers from `src/`. Keep steps thin — parse the argument, call one
//! helper, assert. Business logic in a step definition is logic the unit tests
//! and the coverage gate never see.
//!
//! This target sets `harness = false` in `Cargo.toml` because cucumber brings
//! its own runner; `fn main` at the bottom is what `cargo test` executes.

use cucumber::{World, given, then, when};

/// State carried between the steps of a single scenario.
#[derive(Debug, Default, World)]
struct CliWorld {
    /// Arguments the invocation was given, after the program name.
    args: Vec<String>,
    /// Text produced by the most recent render step.
    rendered: Option<String>,
}

impl CliWorld {
    /// Returns the text rendered earlier in the scenario.
    ///
    /// # Panics
    ///
    /// Panics if no render step ran first — a scenario wiring mistake, which
    /// should fail loudly rather than assert against an empty string.
    fn rendered(&self) -> &str {
        self.rendered
            .as_deref()
            .expect("a `When` step must render the output before a `Then` step inspects it")
    }
}

#[given(expr = "the command line arguments {string}")]
fn given_arguments(world: &mut CliWorld, args: String) {
    world.args = args.split_whitespace().map(ToOwned::to_owned).collect();
}

#[given("no command line arguments")]
fn given_no_arguments(world: &mut CliWorld) {
    world.args.clear();
}

#[when("the output is rendered")]
fn render_output(world: &mut CliWorld) {
    world.rendered = Some(ironroot_cli_app::render(&world.args));
}

#[then(expr = "the output should contain {string}")]
fn output_contains(world: &mut CliWorld, expected: String) {
    let rendered = world.rendered();
    assert!(
        rendered.contains(&expected),
        "expected the output to contain {expected:?}, got:\n{rendered}"
    );
}

#[then(expr = "the output should not contain {string}")]
fn output_does_not_contain(world: &mut CliWorld, forbidden: String) {
    let rendered = world.rendered().to_lowercase();
    assert!(
        !rendered.contains(&forbidden.to_lowercase()),
        "expected the output not to contain {forbidden:?}, got:\n{rendered}"
    );
}

fn main() {
    futures::executor::block_on(CliWorld::run("tests/features"));
}
