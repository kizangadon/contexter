# Acceptance Criteria — Contexter Phase 1R Restructure

All paths relative to repo root unless noted.

## Workspace Structure (Implementation Plan Phase 1)

| ID | Given | When | Then |
|---|---|---|---|
| AC-WS-001 | Repo root `Cargo.toml` | reading | Has `[workspace]` with `members = ["contexter-core"]`, no `[package]` |
| AC-WS-002 | `contexter-core/Cargo.toml` | reading | Has `[package] name = "contexter-core"`, `[lib] name = "contexter_core"`, `[[bin]]` entry |
| AC-WS-003 | The repo root | running `ls` | `contexter-core/` directory exists, `src/` does NOT exist at root |
| AC-WS-004 | The repo root | running `ls` | `docs/` exists, `contexter-core/` exists, no `src/` or `tests/` at root |

## Module Tree (Architecture Spec Section 4.1)

| ID | Given | When | Then |
|---|---|---|---|
| AC-MOD-001 | `contexter-core/src/` directory | listing | Directories `models/`, `engine/`, `storage/`, `cache/`, `compression/`, `wal/`, `telemetry/`, `crdt/`, `versioning/`, `util/`, `vector/`, `fts/`, `analytics/` all exist |
| AC-MOD-002 | `contexter-core/src/storage/` | listing | Files `mod.rs`, `rocksdb.rs`, `column_families.rs`, `migrations.rs`, `types.rs` exist |
| AC-MOD-003 | `contexter-core/src/cache/` | listing | Files `mod.rs`, `dashmap_lru.rs`, `metrics.rs` exist |
| AC-MOD-004 | `contexter-core/src/compression/` | listing | Files `mod.rs`, `codecs.rs` exist |
| AC-MOD-005 | `contexter-core/src/engine/` | listing | Files `mod.rs`, `session.rs`, `memory.rs`, `agent.rs`, `skill.rs`, `search.rs`, `export.rs`, `analytics.rs` exist |
| AC-MOD-006 | `contexter-core/src/bridge.rs` | checking | All `#[pyclass]` and `#[pymethods]` declarations are in `bridge.rs` |
| AC-MOD-007 | `contexter-core/src/wal/` | checking | `mod.rs` exists with RocksDB WAL wrapper |
| AC-MOD-008 | `contexter-core/src/telemetry/` | checking | `mod.rs`, `metrics.rs`, `reporter.rs` exist |
| AC-MOD-009 | `contexter-core/src/crdt/` | checking | `mod.rs`, `merge.rs` exist |
| AC-MOD-010 | `contexter-core/src/versioning/` | checking | `mod.rs`, `store.rs`, `gc.rs`, `diff.rs` exist |
| AC-MOD-011 | `contexter-core/src/util/` | checking | `mod.rs`, `id.rs`, `time.rs` exist |

## Per-Entity DDD Models (Architecture Spec Section 5.2)

| ID | Given | When | Then |
|---|---|---|---|
| AC-MDL-001 | `contexter-core/src/models/memory.rs` | reading | `Memory` struct has fields: id, session_id, agent_id, type, content, embedding, tags, version, created_at, updated_at |
| AC-MDL-002 | `contexter-core/src/models/session.rs` | reading | `Session` struct has fields: id, project, agent_id, status, turn_count, duration_ms, efficiency_score, metadata, created_at, last_active |
| AC-MDL-003 | `contexter-core/src/models/agent.rs` | reading | `Agent` struct has fields: id, name, type, description, capabilities, status, config, version, created_at, updated_at |
| AC-MDL-004 | `contexter-core/src/models/skill.rs` | reading | `Skill` struct has fields: id, name, description, category, version, file_path, created_at, updated_at |
| AC-MDL-005 | `contexter-core/src/models/settings.rs` | reading | Settings types exist (transferred from types/mod.rs) |
| AC-MDL-006 | `contexter-core/src/models/audit.rs` | reading | `AuditEntry` struct has fields: id, entity_type, entity_id, action, actor, summary, metadata, created_at |
| AC-MDL-007 | `contexter-core/src/models/telemetry.rs` | reading | `TelemetryEvent` struct has fields: id, event_type, scope, value, labels, timestamp |
| AC-MDL-008 | `contexter-core/src/models/notification.rs` | reading | Notification entity exists |
| AC-MDL-009 | `contexter-core/src/models/feedback.rs` | reading | Feedback entity exists |
| AC-MDL-010 | `contexter-core/src/models/correlation.rs` | reading | Correlation types exist |
| AC-MDL-011 | `contexter-core/src/models/analytics.rs` | reading | Analytics aggregation types exist |
| AC-MDL-012 | `contexter-core/src/models/mod.rs` | reading | All entity types re-exported with `pub use` |

## StorageBackend Trait (Architecture Spec Section 6.1)

| ID | Given | When | Then |
|---|---|---|---|
| AC-TRB-001 | `contexter-core/src/storage/mod.rs` trait definition | checking methods | `StorageBackend` has ALL 34 methods from Section 6.1 |
| AC-TRB-002 | The trait method list | scanning | `index_embedding`, `knn_search`, `fts_index`, `fts_search`, `replay_wal_since` are present |
| AC-TRB-003 | `RocksDbBackend` | checking | Implements all 34 trait methods |
| AC-TRB-004 | Each stub method | checking body | Uses `unimplemented!("...Phase 2...")` — not `panic!()` or compile error |

## PyO3 Bridge (Architecture Spec Section 7)

| ID | Given | When | Then |
|---|---|---|---|
| AC-BRG-001 | `contexter-core/src/bridge.rs` | checking | `Engine` `#[pyclass]` exists with session/memory methods |
| AC-BRG-002 | Engine methods | checking | `store(&self, cf: &str, key: &str, value: &str) -> PyResult<()>` exists |
| AC-BRG-003 | Engine methods | checking | `get(&self, cf: &str, key: &str) -> PyResult<Option<String>>` exists |
| AC-BRG-004 | `contexter-core/src/lib.rs` | checking | `pub mod bridge` declared, `pub mod python` removed |

## Test Structure (Architecture Spec Section 13)

| ID | Given | When | Then |
|---|---|---|---|
| AC-TST-001 | `contexter-core/tests/` directory | listing | Dirs `storage/`, `cache/`, `compression/`, `engine/`, `bridges/`, `common/` exist |
| AC-TST-002 | `contexter-core/tests/storage/` | listing | `rocksdb_test.rs` exists with RocksDB lifecycle tests |
| AC-TST-003 | `contexter-core/tests/cache/lru_test.rs` | checking | Contains cache eviction/concurrency tests |
| AC-TST-004 | `contexter-core/tests/engine/session_test.rs` | checking | Contains session lifecycle tests |
| AC-TST-005 | `contexter-core/tests/engine/memory_test.rs` | checking | Contains memory CRUD tests |
| AC-TST-006 | `contexter-core/tests/compression/codecs_test.rs` | checking | Contains Zstd/LZ4 round-trip tests |
| AC-TST-007 | `contexter-core/tests/bridges/pyo3_test.rs` | checking | Contains PyO3 type mapping tests |
| AC-TST-008 | `contexter-core/tests/common/mod.rs` | checking | Provides `TempRocksDb::new()` and sample data generators |
| AC-TST-009 | Every `.rs` in `contexter-core/src/` | checking | Has `#[cfg(test)] mod tests { ... }` with at least one test |

## Build & Test

| ID | Given | When | Then |
|---|---|---|---|
| AC-BLD-001 | The repo root | running `cargo build` | Build succeeds with no errors |
| AC-BLD-002 | The repo root | running `cargo clippy` | No new warnings (pre-existing only) |
| AC-BLD-003 | The repo root | running `cargo test --workspace` | All tests pass, count ≥ previous count |
| AC-BLD-004 | `contexter-core/Cargo.toml` | checking | `similar` dependency added |

## Key Encoding

| ID | Given | When | Then |
|---|---|---|---|
| AC-KEY-001 | `contexter-core/src/storage/column_families.rs` | checking | Key encoding/decoding functions are in this file |
| AC-KEY-002 | Key prefixes | checking | `mem:`, `ses:`, `agt:`, `skl:`, `cfg:`, `aud:` are used (unchanged) |
