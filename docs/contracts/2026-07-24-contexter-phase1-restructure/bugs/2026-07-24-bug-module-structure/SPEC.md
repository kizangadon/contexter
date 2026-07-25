# Bug 9: Module Structure

## REQ-MOD-001: Convert error.rs to error/mod.rs
Convert `src/error.rs` from a flat file to `src/error/mod.rs` with `EngineError` + `EngineResult` + `From` impls. All imports across the codebase must be updated from `crate::error::*` to `crate::error::*` (same import path since `pub mod error` in lib.rs remains unchanged).

## REQ-MOD-002: Convert cli.rs to cli/mod.rs
Convert `src/cli.rs` from a flat file to `src/cli/mod.rs`. Same import preservation principle.

## REQ-MOD-003: Replace glob re-export with explicit re-exports
In `src/lib.rs`, replace `pub use models::*;` with explicit re-exports of every type that's actually used from the `models` module. List the exact types.
