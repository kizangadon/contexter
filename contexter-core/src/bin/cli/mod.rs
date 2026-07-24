//! Binary entry point for the Contexter CLI.
//!
//! Delegates to `contexter_core::cli::main()` for all command parsing
//! and dispatch logic.

fn main() {
    contexter_core::cli::main();
}

#[cfg(test)]
mod tests {
    /// Verify that the binary entry point module compiles.
    /// main() delegates to cli::main() — full coverage in cli.rs tests.
    #[test]
    fn test_binary_entry_compiles() {
        // Compilation check: this test exists to ensure bin/cli.rs is
        // exercised during `cargo test`. The actual CLI logic is tested
        // in contexter_core::cli tests.
    }
}
