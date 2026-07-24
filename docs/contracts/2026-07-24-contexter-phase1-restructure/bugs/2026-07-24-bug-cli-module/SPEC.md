# Bug 22: Convert cli.rs to cli/mod.rs Directory Module

## REQ-CLI-001: Convert flat cli.rs to cli/mod.rs
Convert `contexter-core/src/cli.rs` (flat file, ~1700 lines) to `contexter-core/src/cli/mod.rs`. All existing sub-`use crate::cli::*` imports must continue to compile unmodified since `pub mod cli;` in `lib.rs` remains unchanged.

Create the directory structure: `contexter-core/src/cli/mod.rs`
