//! Binary entry point for the IronRoot desktop application template.
//!
//! Deliberately trivial: everything worth testing lives in
//! [`ironroot_desktop_app`], the library target next door. See `src/lib.rs` for
//! the rationale and for the TODO list that turns this template into a real
//! windowed application.

fn main() {
    println!("{}", ironroot_desktop_app::banner());
}
