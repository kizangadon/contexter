# Contexter Phase 1R — Rust Core Restructure

> **Status:** `✅ APPROVED` · **Version:** `v2.0.0`
> **Canonical Reference:** [`docs/design/specs/2026-07-23-contexter-system-architecture.md`](../../../design/specs/2026-07-23-contexter-system-architecture.md)
> **Implementation Plan:** [`docs/design/plans/2026-07-23-contexter-implementation-plan.md`](../../../design/plans/2026-07-23-contexter-implementation-plan.md)

---

## Navigation

- [Problem](#problem)
- [Options](#options)
- [Module Tree](#modtree)
- [Entities](#entities)
- [StorageBackend Trait](#trait)
- [Test Structure](#tests)
- [Workspace Layout](#workspace)
- [Questions](#questions)
- [Decisions](#decisions)
- [Scope](#scope)
- [AC](#ac)
- [Edge Cases](#edgecases)
- [Summary](#summary)

---

## Why This Restructure Exists {#problem}

| The Pain | The Principle |
|---|---|
| Current `src/types/mod.rs` (at repo root) is a DDD anti-pattern — all entities in one monolithic file. `src/engine/mod.rs` is 1500+ lines of monolithic dispatch. `tests/integration_test.rs` (at repo root) buries module-specific tests in a single file. `src/` and `tests/` live at the repo root instead of under `contexter-core/` as the spec requires. | The architecture spec (Section 4.1) shows `contexter-core/` as the crate root with `src/` inside it. The implementation plan (Task 1.1) lists `contexter-core/Cargo.toml`, `contexter-core/src/lib.rs`, `contexter-core/tests/...`. The repo must become a workspace with `contexter-core/`, `contexter-server/`, `contexter-web/` as members. Every entity gets its own `models/*.rs` file (DDD per bounded context). Tests mirror source structure (Section 13). |

---

## Design Options {#options}

### Option A — In-Place Restructuring (Chosen)

Create `contexter-core/` subdirectory, move crate inside, split monolith modules, add missing modules, restructure tests. Code logic unchanged.

| Advantages | Disadvantages |
|---|---|
| ✅ All existing tests pass after restructure | ❌ Requires careful import dependency tracking |
| ✅ No regression risk — logic preserved | ❌ Multiple `cargo test` checkpoints needed |
| ✅ Delivers immediately | ❌ Stub modules need `unimplemented!()` markers |
| ✅ Matches spec 1:1 on tree + workspace | |
| ✅ Enables `contexter-server/` and `contexter-web/` later | |

### Option B — Rewrite from Scratch

| Advantages | Disadvantages |
|---|---|
| ❌ | Rejected — working code discarded, weeks of effort |

### Option C — Partial Restructure

| Advantages | Disadvantages |
|---|---|
| ❌ | Module tree still mismatches, workspace still flat, Phase 2 team finds wrong layout |

**Decision:** Option A — In-Place Restructuring.

---

## Workspace Layout {#workspace}

```
/home/don/Code/contexter/          ← Workspace root
├── Cargo.toml                     ← [workspace] members = ["contexter-core"]
├── contexter-core/                ← Rust crate (moved from repo root)
│   ├── Cargo.toml                 ← [package] name = "contexter-core", version = "0.1.0"
│   ├── src/
│   │   ├── lib.rs                 ← Module declarations + re-exports
│   │   ├── bridge.rs              ← #[pyclass] Engine + #[pymethods]
│   │   ├── bin/cli.rs             ← Binary entry point (unchanged)
│   │   ├── error/mod.rs           ← EngineError (unchanged)
│   │   ├── cli/mod.rs             ← CLI modules (unchanged)
│   │   ├── models/                ★ DDD per-entity (was types/)
│   │   ├── engine/                ★ Domain operations (was monolithic mod.rs)
│   │   ├── storage/               ★ Multi-file (was single rocksdb_backend.rs)
│   │   ├── cache/                 ★ Multi-file (was monolithic mod.rs)
│   │   ├── compression/           ★ Multi-file (was monolithic mod.rs)
│   │   ├── wal/                   ★ NEW — RocksDB WAL wrapper
│   │   ├── telemetry/             ★ NEW — Self-observability
│   │   ├── crdt/                  ★ NEW — LWW-Register
│   │   ├── versioning/            ★ NEW — Content-addressed store
│   │   ├── util/                  ★ NEW — UUID + time helpers
│   │   ├── vector/                ★ NEW — Phase 2 stub
│   │   ├── fts/                   ★ NEW — Phase 2 stub
│   │   └── analytics/             ★ NEW — Phase 2 stub
│   │
│   ├── tests/                     ★ Mirrors src/ (was single integration_test.rs)
│   │   ├── common/
│   │   ├── storage/
│   │   ├── cache/
│   │   ├── compression/
│   │   ├── engine/
│   │   └── bridges/
│   │
│   └── Cargo.lock                 (generated, at workspace root via Cargo workspace)
│
├── contexter-server/              ← (future — Python FastAPI layer)
├── contexter-web/                 ← (future — React SPA)
├── docs/
│   ├── design/
│   │   ├── specs/
│   │   └── plans/
├── python/                        ← Existing Python scripts (unchanged)
└── README.md                      ← At repo root (unchanged)
```

---

## Module Tree {#modtree}

```
contexter-core/src/
├── lib.rs              models/         engine/         storage/
├── bridge.rs            ├── memory      ├── mod          ├── mod (trait)
├── error/mod            ├── session     ├── session      ├── rocksdb
├── cli/mod              ├── agent       ├── memory       ├── column_families
                         ├── skill       ├── agent        ├── migrations
                         ├── settings    ├── skill        └── types
                         ├── audit       ├── search
                         ├── telemetry   ├── export       cache/
                         ├── notification└── analytics    ├── mod
                         ├── feedback                    ├── dashmap_lru
                         ├── correlation                  └── metrics
                         └── analytics
                                        compression/
 wal/      telemetry/    crdt/            ├── mod
 ├── mod    ├── mod       ├── mod          └── codecs
           ├── metrics    └── merge
           ├── reporter                 versioning/
           └── tracing                   ├── mod
                                          ├── store      util/
 vector/    fts/       analytics/          ├── gc          ├── mod
 ├── mod    ├── mod     ├── mod            └── diff        ├── id
 (stub)     (stub)      (stub)                             └── time
```

---

## Entities {#entities}

All entities from Section 5.2, each in its own `models/*.rs` file:

| Entity | File | Key Fields |
|---|---|---|
| **Session** | `contexter-core/src/models/session.rs` | id, project, agent_id, status, turn_count, duration_ms, efficiency_score, metadata, created_at, last_active |
| **Memory** | `contexter-core/src/models/memory.rs` | id, session_id, agent_id, type, content, embedding, tags, version, created_at, updated_at |
| **Agent** | `contexter-core/src/models/agent.rs` | id, name, type, description, capabilities, status, config, version, created_at, updated_at |
| **Skill** | `contexter-core/src/models/skill.rs` | id, name, description, category, version, file_path, created_at, updated_at |
| **Settings** | `contexter-core/src/models/settings.rs` | Config key-value types |
| **AuditEntry** | `contexter-core/src/models/audit.rs` | id, entity_type, entity_id, action, actor, summary, metadata, created_at |
| **TelemetryEvent** | `contexter-core/src/models/telemetry.rs` | id, event_type, scope, value, labels, timestamp |
| **Notification** | `contexter-core/src/models/notification.rs` | Notification record |
| **Feedback** | `contexter-core/src/models/feedback.rs` | Bug report/suggestion |
| **Correlation** | `contexter-core/src/models/correlation.rs` | Cross-session correlation |
| **Analytics** | `contexter-core/src/models/analytics.rs` | Aggregated analytics |

(Current `types/mod.rs` content is split across model files; existing types re-exported via `models/mod.rs` and `lib.rs`.)

---

## StorageBackend Trait — All 34 Methods {#trait}

Defined in `contexter-core/src/storage/mod.rs`:

```rust
pub trait StorageBackend: Send + Sync {
    // Session (6)
    fn create_session(&self, session: NewSession) -> Result<Session>;
    fn get_session(&self, id: Uuid) -> Result<Option<Session>>;
    fn list_sessions(&self, filter: &SessionFilter) -> Result<Vec<Session>>;
    fn update_session(&self, id: Uuid, patch: &SessionPatch) -> Result<Session>;
    fn delete_session(&self, id: Uuid) -> Result<()>;
    fn count_sessions(&self, filter: &SessionFilter) -> Result<u64>;

    // Memory (6)
    fn create_memory(&self, memory: NewMemory) -> Result<Memory>;
    fn get_memory(&self, id: Uuid) -> Result<Option<Memory>>;
    fn search_memories(&self, query: &MemorySearchQuery) -> Result<Vec<Memory>>;
    fn update_memory(&self, id: Uuid, patch: &MemoryPatch) -> Result<Memory>;
    fn delete_memory(&self, id: Uuid) -> Result<()>;
    fn count_memories(&self, filter: &MemoryFilter) -> Result<u64>;

    // Agent (5)
    fn create_agent(&self, agent: NewAgent) -> Result<Agent>;
    fn get_agent(&self, id: Uuid) -> Result<Option<Agent>>;
    fn list_agents(&self, filter: &AgentFilter) -> Result<Vec<Agent>>;
    fn update_agent(&self, id: Uuid, patch: &AgentPatch) -> Result<Agent>;
    fn delete_agent(&self, id: Uuid) -> Result<()>;

    // Skill (5)
    fn create_skill(&self, skill: NewSkill) -> Result<Skill>;
    fn get_skill(&self, id: Uuid) -> Result<Option<Skill>>;
    fn list_skills(&self, filter: &SkillFilter) -> Result<Vec<Skill>>;
    fn update_skill(&self, id: Uuid, patch: &SkillPatch) -> Result<Skill>;
    fn delete_skill(&self, id: Uuid) -> Result<()>;

    // Vector — Phase 2 stubs (2)
    fn index_embedding(&self, memory_id: Uuid, embedding: &[f32]) -> Result<()>;
    fn knn_search(&self, query: &[f32], k: usize, filter: &VectorFilter) -> Result<Vec<ScoredMemoryId>>;

    // Full-Text Search — Phase 2 stubs (2)
    fn fts_index(&self, memory_id: Uuid, content: &str, tags: &[String]) -> Result<()>;
    fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<ScoredMemoryId>>;

    // Settings (2)
    fn get_setting(&self, key: &str) -> Result<Option<String>>;
    fn set_setting(&self, key: &str, value: &str) -> Result<()>;

    // Audit (2)
    fn append_audit_entry(&self, entry: &NewAuditEntry) -> Result<()>;
    fn query_audit_log(&self, filter: &AuditFilter) -> Result<Vec<AuditEntry>>;

    // Maintenance (3)
    fn flush(&self) -> Result<()>;
    fn checkpoint(&self) -> Result<u64>;
    fn replay_wal_since(&self, lsn: u64) -> Result<Vec<WalRecord>>;
    fn storage_size(&self) -> Result<StorageSize>;
}
```

**5 new methods (currently missing):** `index_embedding`, `knn_search`, `fts_index`, `fts_search`, `replay_wal_since`

Stub implementations use `unimplemented!("... Phase 2 ...")`.

---

## Test Structure {#tests}

Per Section 13 of architecture spec. Tests mirror source structure:

```
contexter-core/tests/               ← Mirrors contexter-core/src/
├── common/
│   ├── mod.rs                     ← TempRocksDb::new(), sample data generators
│   └── fixtures.rs                ← Reusable test data constants
├── storage/
│   ├── rocksdb_test.rs            ← Full CRUD, WAL replay (moved from integration_test.rs)
│   └── column_families_test.rs
├── cache/
│   └── lru_test.rs                ← Eviction, concurrency (moved)
├── compression/
│   └── codecs_test.rs             ← Zstd/LZ4 round-trip (moved)
├── engine/
│   ├── memory_test.rs             ← Full lifecycle via Engine (moved)
│   ├── session_test.rs            ← Session lifecycle (moved)
│   └── search_test.rs
└── bridges/
    └── pyo3_test.rs               ← PyO3 type mapping, JSON round-trip (moved from python.rs)
```

Every source `.rs` file also has inline `#[cfg(test)] mod tests { ... }`.

---

## Open Questions {#questions}

| ID | Question | Status |
|---|---|---|
| OQ-001 | Should `vector/`, `fts/`, `analytics/` stubs be in directories or TODO comments? | ✅ Resolved — stub `mod.rs` per spec |
| OQ-002 | Should `similar` be unconditional or feature-gated? | ✅ Resolved — unconditional dependency |
| OQ-003 | Should `python.rs` rename to `bridge.rs` or stay as-is? | ✅ Resolved — content moves to `bridge.rs` |
| OQ-004 | Phase 2 stub methods — panic or return Err? | ✅ Resolved — `EngineError::Unimplemented` |
| OQ-005 | Should old `src/` and `tests/` at repo root be deleted? | ✅ Resolved — yes, contents move to `contexter-core/` |
| OQ-006 | Workspace or standalone crate? | ✅ Resolved — workspace with `contexter-core` member, enabling future `contexter-server/` and `contexter-web/` |

---

## Decision Log {#decisions}

| Date | ID | Description | Rationale |
|---|---|---|---|
| 2026-07-24 | CON-001 | In-place restructuring, not rewrite | Preserves working code/tests; mechanical code movement |
| 2026-07-24 | CON-002 | Stub methods return `EngineError::Unimplemented` | Callers handle gracefully vs panic |
| 2026-07-24 | CON-003 | `similar` is unconditional dependency | Required by `versioning/diff.rs` — lightweight |
| 2026-07-24 | CON-004 | Phase 2 modules get stub `mod.rs` | Tree matches spec; no orphan module warnings |
| 2026-07-24 | CON-005 | Workspace Cargo.toml at repo root | Enables future `contexter-server/`, `contexter-web/` |
| 2026-07-24 | CON-006 | `contexter-core/` as crate root with `src/` inside | Matches implementation plan Task 1.1 files |
| 2026-07-24 | CON-007 | Tests split, not deleted | Integration tests preserved per domain |

---

## Out of Scope {#scope}

| # | Item | Rationale |
|---|---|---|
| 01 | Phase 2 features (L3 HNSW, L4 Tantivy, L5 DuckDB) | Stub methods only |
| 02 | Python layer (`contexter-server/`) | Future workspace member |
| 03 | React UI (`contexter-web/`) | Future workspace member |
| 04 | Adding new business logic | Restructuring only |
| 05 | Benchmarking or performance tuning | Phase 2 |

---

## Acceptance Criteria {#ac}

> **Status:** 46 Pending

| Group | Count | Key Verifications |
|---|---|---|
| AC-WS (Workspace) | 4 | `cargo build` passes, `src/` not at root, `workspace` in root Cargo.toml |
| AC-MOD (Module Tree) | 11 | All 12 module dirs exist under `contexter-core/src/` |
| AC-MDL (DDD Entities) | 12 | 11 entity files + re-exports |
| AC-TRB (StorageBackend) | 4 | 34 methods, 5 new, all implemented/stubbed |
| AC-BRG (PyO3 Bridge) | 4 | `store`/`get` on Engine |
| AC-TST (Tests) | 9 | Test dirs mirror src/, TempRocksDb helper |
| AC-BLD (Build & Test) | 4 | `cargo build`, `cargo clippy`, `cargo test` |
| AC-KEY (Key Encoding) | 2 | Prefixes match spec |

---

## Edge Cases {#edgecases}

> **Status:** 22 Identified

| Area | Count | Key Risks |
|---|---|---|
| Workspace & Crate Relocation | 4 | Cargo.lock conflict, IDE tooling paths |
| Module Restructuring | 5 | Cyclic imports, stale `types/` dir |
| StorageBackend Trait | 4 | Unimplemented variant missing |
| Bridge | 2 | Python import paths |
| Test Migration | 3 | `crate::types` path updates, test count drop |
| Dependencies | 2 | `similar` missing, old Cargo.toml with `[package]` |
| Build | 2 | Dead code in stubs |

---

## Design Draft Summary {#summary}

| Metric | Count |
|---|---|
| Acceptance Criteria | 46 |
| Edge Cases | 22 |
| Modules to create/split | 12 (under `contexter-core/src/`) |
| New test modules | 6 (under `contexter-core/tests/`) |
| Trait methods added | 5 |
| Cargo dependency changes | 1 (`similar`) |
| Files moved/restructured | ~45 |
| Stub modules (Phase 2) | 3 (vector, fts, analytics) |

---

**Generated · 2026-07-24 · Contexter Phase 1R — Rust Core Restructure · v2.0.0-draft**
