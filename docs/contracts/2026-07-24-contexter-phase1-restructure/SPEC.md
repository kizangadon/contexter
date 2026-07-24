---
title: Contexter Phase 1R — Rust Core Restructure & Realignment
version: 2.0
date_created: 2026-07-24
owner: Orchestrator
tags: contexter, phase1, restructure, ddd, rust
---

# Introduction

Restructure the existing `contexter-core` Rust implementation to match the approved architecture specification at `docs/design/specs/2026-07-23-contexter-system-architecture.md` (Section 4.1 for module tree, 5.2 for entities, 6.1 for StorageBackend trait, 7 for PyO3 bridge, 13 for test structure) and the implementation plan at `docs/design/plans/2026-07-23-contexter-implementation-plan.md` (Phase 1 task file listings).

The current implementation has a flat structure with `src/` and `tests/` **at the repo root**. The spec and plan require them to be **under `contexter-core/`** as a workspace member crate. The module layout also does not match the spec — monolithic `types/mod.rs` instead of per-entity `models/` files, monolithic `engine/mod.rs` instead of domain-split files, missing modules (wal, telemetry, crdt, versioning, util, vector, fts, analytics), and flat `tests/integration_test.rs` instead of `tests/` mirroring `src/`.

This is an **in-place restructuring** — existing business logic, tests, and behavior are preserved. Code is moved, split, and reorganized, not rewritten.

## 1. Purpose & Scope

This specification defines the restructuring of `contexter-core` to match the approved architecture:
- Move the Rust crate from repo root into `contexter-core/` subdirectory (workspace member)
- Restructure module tree inside `contexter-core/src/` per Section 4.1
- Add per-entity DDD model files inside `contexter-core/src/models/` per Section 5.2
- Split monolithic modules into multi-file modules
- Add missing methods to `StorageBackend` trait per Section 6.1
- Add generic `store`/`get` to Engine per Section 7
- Restructure tests under `contexter-core/tests/` to mirror `src/` per Section 13

**Target audience:** Distinguished Full Stack Engineer implementing the restructure.
**Assumptions:** The architecture spec at `docs/design/specs/2026-07-23-contexter-system-architecture.md` is the canonical reference. The implementation plan at `docs/design/plans/2026-07-23-contexter-implementation-plan.md` defines task file listings.

## 2. Definitions

| Term | Definition |
|---|---|
| Architecture Spec | `docs/design/specs/2026-07-23-contexter-system-architecture.md` |
| Implementation Plan | `docs/design/plans/2026-07-23-contexter-implementation-plan.md` |
| DDD | Domain-Driven Design — business logic in per-domain entities, not monolithic modules |
| CF | Column Family (RocksDB) |
| WAL | Write-Ahead Log (RocksDB built-in) |
| LWW-Register | Last-Writer-Wins CRDT for conflict resolution |

## 3. Requirements

### REQ-WS: Workspace Structure (Implementation Plan Phase 1)

- **REQ-WS-001**: A workspace `Cargo.toml` SHALL exist at the repo root (`/home/don/Code/contexter/Cargo.toml`) with `[workspace] members = ["contexter-core"]` — the workspace SHALL NOT have a `[package]` section
- **REQ-WS-002**: `contexter-core/Cargo.toml` SHALL contain the package definition (moved from repo root)
- **REQ-WS-003**: All `src/` content SHALL move to `contexter-core/src/`
- **REQ-WS-004**: All `tests/` content SHALL move to `contexter-core/tests/`
- **REQ-WS-005**: `contexter-core/` SHALL be a standard Cargo crate with `[lib]` and `[[bin]]` entries matching current configuration
- **REQ-WS-006**: `cargo build` from repo root SHALL succeed (workspace delegates to `contexter-core/`)

### REQ-MODULE: Module Structure (Architecture Spec Section 4.1)

- **REQ-MOD-001**: `contexter-core/src/lib.rs` SHALL export all public modules as defined in Section 4.1
- **REQ-MOD-002**: `contexter-core/src/bridge.rs` SHALL contain all `#[pyclass]` and `#[pymethods]` declarations (moved from `src/python.rs`)
- **REQ-MOD-003**: `contexter-core/src/models/` SHALL replace `src/types/` with per-entity files (see REQ-ENT)
- **REQ-MOD-004**: `contexter-core/src/models/mod.rs` SHALL re-export all entity types
- **REQ-MOD-005**: `contexter-core/src/storage/` SHALL be split into `mod.rs`, `rocksdb.rs`, `column_families.rs`, `migrations.rs`, `types.rs`
- **REQ-MOD-006**: `contexter-core/src/cache/` SHALL be split into `mod.rs`, `dashmap_lru.rs`, `metrics.rs`
- **REQ-MOD-007**: `contexter-core/src/compression/` SHALL be split into `mod.rs`, `codecs.rs`
- **REQ-MOD-008**: `contexter-core/src/engine/` SHALL be split into `mod.rs`, `session.rs`, `memory.rs`, `agent.rs`, `skill.rs`, `search.rs`, `export.rs`, `analytics.rs`
- **REQ-MOD-009**: `contexter-core/src/wal/` SHALL be created as a thin wrapper over RocksDB built-in WAL
- **REQ-MOD-010**: `contexter-core/src/telemetry/` SHALL be created with `mod.rs`, `metrics.rs`, `reporter.rs`, `tracing.rs`
- **REQ-MOD-011**: `contexter-core/src/crdt/` SHALL be created with `mod.rs` (LWW-Register), `merge.rs`
- **REQ-MOD-012**: `contexter-core/src/versioning/` SHALL be created with `mod.rs` (ContentAddressedStore), `store.rs`, `gc.rs`, `diff.rs`
- **REQ-MOD-013**: `contexter-core/src/util/` SHALL be created with `mod.rs`, `id.rs`, `time.rs`
- **REQ-MOD-014**: `contexter-core/src/vector/`, `contexter-core/src/fts/`, `contexter-core/src/analytics/` module directories SHALL be created stub `mod.rs` files (structural placeholders for Phase 2)

### REQ-ENTITY: Per-Entity DDD Models (Architecture Spec Section 5.2)

- **REQ-ENT-001**: `contexter-core/src/models/memory.rs` SHALL contain `Memory` entity with all fields from Section 5.2
- **REQ-ENT-002**: `contexter-core/src/models/session.rs` SHALL contain `Session` entity with all fields from Section 5.2
- **REQ-ENT-003**: `contexter-core/src/models/agent.rs` SHALL contain `Agent` entity with all fields from Section 5.2
- **REQ-ENT-004**: `contexter-core/src/models/skill.rs` SHALL contain `Skill` entity with all fields from Section 5.2
- **REQ-ENT-005**: `contexter-core/src/models/settings.rs` SHALL contain settings types
- **REQ-ENT-006**: `contexter-core/src/models/audit.rs` SHALL contain `AuditEntry` entity with fields from Section 5.2
- **REQ-ENT-007**: `contexter-core/src/models/telemetry.rs` SHALL contain `TelemetryEvent` entity with fields from Section 5.2
- **REQ-ENT-008**: `contexter-core/src/models/notification.rs` SHALL contain `Notification` entity
- **REQ-ENT-009**: `contexter-core/src/models/feedback.rs` SHALL contain `Feedback` entity
- **REQ-ENT-010**: `contexter-core/src/models/correlation.rs` SHALL contain correlation types
- **REQ-ENT-011**: `contexter-core/src/models/analytics.rs` SHALL contain analytics aggregation types
- **REQ-ENT-012**: `contexter-core/src/models/mod.rs` SHALL re-export all entity types with `pub use`

### REQ-TRAIT: StorageBackend Trait (Architecture Spec Section 6.1)

- **REQ-TRB-001**: `StorageBackend` trait SHALL be defined in `contexter-core/src/storage/mod.rs`
- **REQ-TRB-002**: ALL 34 methods from Section 6.1 SHALL be present in the trait, including the 5 currently missing:
  - `index_embedding(&self, memory_id: Uuid, embedding: &[f32]) -> Result<()>`
  - `knn_search(&self, query: &[f32], k: usize, filter: &VectorFilter) -> Result<Vec<ScoredMemoryId>>`
  - `fts_index(&self, memory_id: Uuid, content: &str, tags: &[String]) -> Result<()>`
  - `fts_search(&self, query: &str, limit: usize) -> Result<Vec<ScoredMemoryId>>`
  - `replay_wal_since(&self, lsn: u64) -> Result<Vec<WalRecord>>`
- **REQ-TRB-003**: The `RocksDbBackend` implementation SHALL implement ALL 34 trait methods
- **REQ-TRB-004**: Missing methods (vector, FTS, WAL replay) SHALL use `unimplemented!()` with a tracking message — they are Phase 2
- **REQ-TRB-005**: Tests SHALL exist for each trait method

### REQ-BRIDGE: PyO3 Bridge (Architecture Spec Section 7)

- **REQ-BRG-001**: `contexter-core/src/bridge.rs` SHALL contain the `Engine` `#[pyclass]` with all `#[pymethods]`
- **REQ-BRG-002**: Generic `store(&self, cf: &str, key: &str, value: &str) -> PyResult<()>` SHALL be added
- **REQ-BRG-003**: Generic `get(&self, cf: &str, key: &str) -> PyResult<Option<String>>` SHALL be added
- **REQ-BRG-004**: `src/python.rs` SHALL be replaced/absorbed by `contexter-core/src/bridge.rs`

### REQ-CRDT: CRDT & Versioning (Architecture Spec Section 8)

- **REQ-CRD-001**: `contexter-core/src/crdt/mod.rs` SHALL define LWW-Register with logical + wall clock timestamps
- **REQ-CRD-002**: `contexter-core/src/crdt/merge.rs` SHALL implement conflict resolution (higher timestamp wins, loser preserved)
- **REQ-CRD-003**: `contexter-core/src/versioning/store.rs` SHALL implement SHA-256 content-addressed storage
- **REQ-CRD-004**: `contexter-core/src/versioning/gc.rs` SHALL implement reference counting + sweep
- **REQ-CRD-005**: `contexter-core/src/versioning/diff.rs` SHALL implement line-level diff (via `similar` crate)

### REQ-TEST: Test Structure (Architecture Spec Section 13)

- **REQ-TST-001**: `contexter-core/tests/` directory SHALL mirror `contexter-core/src/` structure per Section 13.1
- **REQ-TST-002**: `contexter-core/tests/storage/` SHALL contain `rocksdb_test.rs`
- **REQ-TST-003**: `contexter-core/tests/cache/` SHALL contain `lru_test.rs`
- **REQ-TST-004**: `contexter-core/tests/compression/` SHALL contain `codecs_test.rs`
- **REQ-TST-005**: `contexter-core/tests/engine/` SHALL contain `session_test.rs`, `memory_test.rs`
- **REQ-TST-006**: `contexter-core/tests/bridges/` SHALL contain `pyo3_test.rs`
- **REQ-TST-007**: `contexter-core/tests/common/mod.rs` SHALL provide `TempRocksDb::new()`, sample data generators
- **REQ-TST-008**: Every `.rs` file under `contexter-core/src/` SHALL have inline `#[cfg(test)] mod tests { ... }`
- **REQ-TST-009**: ALL existing tests SHALL continue to pass after restructuring

### REQ-ENGINE: Engine Generic Methods (Architecture Spec Section 7)

- **REQ-ENG-001**: `Engine` SHALL have `store(cf, key, value)` and `get(cf, key)` generic methods
- **REQ-ENG-002**: Engine composition SHALL include: `cache`, `storage`, `telemetry` (Phase 2 adds `vector_index`, `fts_index`, `analytics`)

## 4. Constraints

- **CON-001**: No existing test behavior SHALL be changed (tests may be moved but not rewritten)
- **CON-002**: All existing public APIs SHALL be preserved (re-exported from new module locations)
- **CON-003**: `contexter-core/Cargo.toml` SHALL be updated to add `similar` crate dependency (for versioning/diff)
- **CON-004**: Key encoding prefixes SHALL NOT be changed (already match spec: `mem:`, `ses:`, `agt:`, `skl:`, `cfg:`, `aud:`)
- **CON-005**: Phase 2 features (vector/L3, FTS/L4, analytics/L5) SHALL use `unimplemented!()` stubs
- **CON-006**: The `vector/`, `fts/`, `analytics/` module directories SHALL exist with stub `mod.rs` files
- **CON-007**: The workspace `Cargo.toml` at repo root SHALL NOT have a `[package]` section — workspace-only

## 5. Path Changes

| Old Path (current) | New Path (target) |
|---|---|
| `Cargo.toml` (package) | `contexter-core/Cargo.toml` + `Cargo.toml` (workspace only) |
| `src/lib.rs` | `contexter-core/src/lib.rs` |
| `src/types/mod.rs` | `contexter-core/src/models/{memory,session,agent,...}.rs` |
| `src/storage/rocksdb_backend.rs` | `contexter-core/src/storage/{rocksdb,column_families,migrations,types}.rs` |
| `src/cache/mod.rs` (monolithic) | `contexter-core/src/cache/{mod,dashmap_lru,metrics}.rs` |
| `src/compression/mod.rs` (monolithic) | `contexter-core/src/compression/{mod,codecs}.rs` |
| `src/engine/mod.rs` (monolithic) | `contexter-core/src/engine/{mod,session,memory,agent,skill,search,export,analytics}.rs` |
| `src/python.rs` | `contexter-core/src/bridge.rs` |
| `tests/integration_test.rs` | `contexter-core/tests/{storage,cache,engine,bridges,compression}/*.rs` |

## 6. Acceptance Criteria

- **AC-001**: `cargo build` from repo root succeeds with no errors
- **AC-002**: `cargo clippy` from repo root is clean (no new warnings)
- **AC-003**: `cargo test` from repo root passes (ALL existing tests, no regressions)
- **AC-004**: Module tree under `contexter-core/src/` matches Section 4.1 (verified by `tree`)
- **AC-005**: All 11 entity types exist as separate `.rs` files under `contexter-core/src/models/`
- **AC-006**: `StorageBackend` trait has all 34 methods from Section 6.1
- **AC-007**: `Engine` has `store(cf, key, value)` and `get(cf, key)` methods
- **AC-008**: `contexter-core/tests/` directory mirrors `contexter-core/src/` structure
- **AC-009**: Workspace `Cargo.toml` at root has no `[package]` section, only `[workspace]`
- **AC-010**: `cargo test --workspace` shows test count ≥ existing count

## 7. Dependencies

- **DEP-001**: `similar = "2"` crate added to `contexter-core/Cargo.toml`
- **DEP-002**: `sha2` already present (no change)
