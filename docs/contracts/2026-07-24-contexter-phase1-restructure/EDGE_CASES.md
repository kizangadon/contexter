# Edge Cases — Contexter Phase 1R Restructure

## Workspace & Crate Relocation

| ID | Scenario | Expected Behavior | Priority |
|---|---|---|---|
| EC-WS-001 | `cargo build` run from repo root with workspace Cargo.toml | Delegates to `contexter-core/`, builds succeeds | High |
| EC-WS-002 | `cargo test` run from `contexter-core/` directory directly | Works (standalone Cargo.toml in `contexter-core/`) | Medium |
| EC-WS-003 | IDE or tooling looks for `src/lib.rs` at repo root | After restructure, `src/` no longer exists at root — tooling must use `contexter-core/` | High |
| EC-WS-004 | `Cargo.lock` at repo root conflicts with `contexter-core/Cargo.lock` | Single `Cargo.lock` at workspace root; delete any inside `contexter-core/` | High |

## Module Restructuring

| ID | Scenario | Expected Behavior | Priority |
|---|---|---|---|
| EC-MOD-001 | `src/types/` (old) still exists after `contexter-core/src/models/` created | `src/types/` MUST be removed; all imports use `contexter-core/src/models/` | High |
| EC-MOD-002 | Cyclic import dependency between new modules | Compiler catches this; restructure MUST design to avoid cycles | High |
| EC-MOD-003 | `src/python.rs` has inline tests mixed with bridge code | Tests move to `contexter-core/tests/bridges/pyo3_test.rs`; bridge code to `contexter-core/src/bridge.rs` | High |
| EC-MOD-004 | `tests/integration_test.rs` has tests for multiple modules | Tests SHALL be split into `contexter-core/tests/storage/`, `tests/engine/`, `tests/cache/`, etc. | High |
| EC-MOD-005 | Stub modules (vector, fts, analytics) have zero content | `mod.rs` exists with `// Placeholder for Phase 2` — compiles clean with `#[allow(dead_code)]` if needed | Medium |

## StorageBackend Trait Methods

| ID | Scenario | Expected Behavior | Priority |
|---|---|---|---|
| EC-TRB-001 | `index_embedding` called before L3 implemented | Returns `EngineError::Unimplemented("Vector index (L3/HNSW) — Phase 2")` | Medium |
| EC-TRB-002 | `fts_search` called before L4 implemented | Returns `EngineError::Unimplemented("Full-text search (L4/Tantivy) — Phase 2")` | Medium |
| EC-TRB-003 | `EngineError::Unimplemented` variant doesn't exist | Must exist for stub methods to compile | High |
| EC-TRB-004 | `replay_wal_since` with invalid LSN | Returns empty vec (RocksDB WAL behavior) | Low |

## Bridge

| ID | Scenario | Expected Behavior | Priority |
|---|---|---|---|
| EC-BRG-001 | `store` called with non-existent CF name | Returns `EngineError::InvalidColumnFamily` | Medium |
| EC-BRG-002 | Python `import contexter_core` after restructure | Bridge module path unchanged (`contexter_core.bridge`) — all `#[pyfn]` entries preserved | High |

## Test Migration

| ID | Scenario | Expected Behavior | Priority |
|---|---|---|---|
| EC-TST-001 | Test file references old `crate::types::*` path | Compiler error — MUST update to `crate::models::*` | High |
| EC-TST-002 | Integration test creates temp RocksDB instance | `contexter-core/tests/common/mod.rs` provides `TempRocksDb::new()` — all tests use shared helper | High |
| EC-TST-003 | Test count drops after restructuring | MUST NOT drop — every test from `tests/integration_test.rs` and inline tests preserved | High |

## Dependencies

| ID | Scenario | Expected Behavior | Priority |
|---|---|---|---|
| EC-DEP-001 | `similar` not added to `contexter-core/Cargo.toml` | `versioning/diff.rs` won't compile | High |
| EC-DEP-002 | Old `Cargo.toml` at repo root still has `[package]` section | Must be removed — workspace `[workspace]` only | High |

## Build

| ID | Scenario | Expected Behavior | Priority |
|---|---|---|---|
| EC-BLD-001 | `vector/`, `fts/`, `analytics/` stub modules compiled with no content | `mod.rs` with empty body + `#[allow(dead_code)]` — compiles clean | Medium |
| EC-BLD-002 | Dead code warnings from unused trait methods in stubs | `#[allow(dead_code)]` on `RocksDbBackend` for Phase 2 methods | Low |
