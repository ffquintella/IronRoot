//! Binary entry point for the IronRoot CLI application template.
//!
//! Deliberately trivial: it collects the arguments and hands them to
//! [`ironroot_cli_app::render`], which is where everything worth testing lives.
//! See `src/lib.rs` for the rationale and for the TODO list that turns this
//! template into a real command-line tool.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    println!("{}", ironroot_cli_app::render(&args));
}
