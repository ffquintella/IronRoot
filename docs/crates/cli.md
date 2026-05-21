# `ironroot-cli`

CLI application abstractions built on top of [`ironroot-core`](core.md).

## What's in it

- `CliApp` — top-level container holding registered commands; owns the
  application name and version.
- `Command` — trait describing a single sub-command (name, description,
  `execute(args)`).

The crate is deliberately argument-parser-agnostic. Concrete integrations
with [`clap`](https://docs.rs/clap) and [`argh`](https://docs.rs/argh) are
planned behind optional feature flags.

## Cargo

```toml
[dependencies]
ironroot-core = "0.2"
ironroot-cli  = "0.2"
```

## Example

```rust
use ironroot_cli::{CliApp, Command};

struct Greet;
impl Command for Greet {
    fn name(&self) -> &str { "greet" }
    fn description(&self) -> &str { "Print a friendly greeting" }
    fn execute(&self, args: &[String]) -> Result<(), String> {
        let who = args.first().map(String::as_str).unwrap_or("world");
        println!("Hello, {who}!");
        Ok(())
    }
}

fn main() {
    let _app = CliApp::new("my-tool", env!("CARGO_PKG_VERSION"));
    // TODO once registration lands:
    // app.register(Greet);
    // app.run(std::env::args().collect::<Vec<_>>().as_slice());
}
```

## Roadmap for this crate

1. `CliApp::register(impl Command)` and `CliApp::run(&[String])` methods.
2. Async variant: `AsyncCommand::execute(&self, args: &[String]) -> impl
   Future<Output = Result<(), String>>`.
3. Optional clap / argh adapters: `features = ["clap"]` etc.
4. Built-in `--help` / `--version` handling using the data already on
   `CliApp`.
