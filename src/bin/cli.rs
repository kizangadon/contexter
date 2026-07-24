//! Binary entry point for the Contexter CLI.
//!
//! Delegates to `contexter_core::cli::main()` for all command parsing
//! and dispatch logic.

fn main() {
    contexter_core::cli::main();
}
