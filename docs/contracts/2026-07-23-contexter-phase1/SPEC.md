---
title: Contexter Phase 1 — Rust Core Foundation
version: 1.0
date_created: 2026-07-23
owner: Engineering
tags: rust, core, storage, rocksdb, pyo3, cli
---

# Introduction

Build the foundational Rust core for Contexter: a RocksDB-backed multi-tier storage engine with a unified PyO3 bridge and CLI diagnostics tool. This phase delivers all entities, session/memory CRUD, and the hot cache tier — everything needed for the Python API layer (Phase 3) to build on.

## 1. Purpose & Scope

**Purpose:** Establish the bottom two layers of the modular monolith — the Rust storage engine (L1 cache + L2 RocksDB) and the PyO3 bridge that exposes it to Python. All subsequent phases (search, API, UI) depend on this foundation.

**Scope:**
- Rust crate `contexter-core` with full module tree and data models
- `StorageBackend` trait with RocksDB implementation (8 column families, per-CF compression)
- L1 hot cache (DashMap + LRU, write-through/write-around)
- `Engine` struct composing cache + storage (with stubs for L3–L5)
- Session CRUD (create, read, list, update, delete)
- Memory CRUD (create, read, search, update, delete)
- PyO3 `#[pyclass] Engine` bridge exposing all operations
- Python `core_bridge.py` async wrapper
- Click CLI for diagnostics
- Zstd/LZ4 compression utilities
- Complete test suite (inline units + integration tests)

**Out of scope (Phase 2+):**
- HNSW vector index (L3)
- Tantivy full-text search (L4)
- DuckDB analytics engine (L5)
- FastAPI REST server
- FastMCP server
- React UI

**Audience:** Engineers implementing Phase 1 and engineers in Phase 2+ who depend on the Engine interface.

## 2. Definitions

| Term | Definition |
|---|---|
| **CF** | Column Family — RocksDB's logical partition within a database, each with independent compression and write buffer settings |
| **CRDT** | Conflict-free Replicated Data Type — here, a Last-Writer-Wins register for concurrent edit resolution |
| **L1** | Hot cache tier (DashMap + LRU) |
| **L2** | Primary storage tier (RocksDB) |
| **LSM** | Log-Structured Merge-tree — RocksDB's underlying data structure |
| **PyO3** | Rust bindings for Python, enabling native Rust modules to be called from Python |
| **StorageBackend** | Rust trait defining the storage interface, implemented by RocksDB (and future backends) |
| **WAL** | Write-Ahead Log — RocksDB's built-in crash recovery mechanism |
| **UUID v7** | Time-ordered UUID format, sortable by creation time |

## 3. Requirements, Constraints & Guidelines

### Storage

- **REQ-S-001**: All entity data MUST be stored in RocksDB with column families as specified in the architecture spec.
- **REQ-S-002**: The `sessions` CF MUST use Zstd compression (level 3) with a 32MB write buffer.
- **REQ-S-003**: The `memory_items` CF MUST use Zstd compression (level 3) with a 64MB write buffer.
- **REQ-S-004**: The `agents` and `skills` CFs MUST use LZ4 compression with 16MB write buffers.
- **REQ-S-005**: The `telemetry` CF MUST use LZ4 compression with a 4MB write buffer (high write throughput).
- **REQ-S-006**: The `efficiency_map` CF MUST use LZ4 compression with an 8MB write buffer.
- **REQ-S-007**: The `conflicts` CF MUST use Zstd compression (level 1) with an 8MB write buffer.
- **REQ-S-008**: The `index_state` CF MUST use LZ4 compression with a 4MB write buffer.
- **REQ-S-009**: RocksDB MUST be configured with `create_if_missing(true)` and `create_missing_column_families(true)`.
- **REQ-S-010**: WAL sync MUST be enabled (`set_sync(true)`) for durability.
- **REQ-S-011**: A 256MB LRU block cache MUST be configured for `memory_items` CF reads.

### Key Encoding

- **REQ-K-001**: Keys MUST follow the pattern `{prefix}:{id}[:{sub_key}]` as defined in the architecture spec.
- **REQ-K-002**: All entity IDs MUST be UUID v7 (time-ordered).
- **REQ-K-003**: The `mem:` prefix MUST route to the `memory_items` CF.
- **REQ-K-004**: The `ses:` prefix MUST route to the `sessions` CF.
- **REQ-K-005**: The `agt:` prefix MUST route to the `agents` CF.
- **REQ-K-006**: The `skl:` prefix MUST route to the `skills` CF.

### StorageBackend Trait

- **REQ-T-001**: `StorageBackend` MUST define all CRUD operations for sessions, memories, agents, skills, settings, and audit.
- **REQ-T-002**: All trait methods MUST be synchronous (not async) — async wrapping is handled at the PyO3 bridge layer.
- **REQ-T-003**: The trait MUST be `Send + Sync`.
- **REQ-T-004**: The RocksDB implementation MUST be behind `Arc<RwLock<Box<dyn StorageBackend>>>`.

### L1 Cache

- **REQ-C-001**: The cache MUST use DashMap for concurrent access + LRU eviction per entity type.
- **REQ-C-002**: Default capacity MUST be 10,000 entries per entity type (configurable).
- **REQ-C-003**: Write-through policy: entity writes go to cache + RocksDB synchronously.
- **REQ-C-004**: Write-around policy: vector writes skip cache (cache invalidated on L3 update via stub).
- **REQ-C-005**: Cache MISS MUST fall through to RocksDB and populate the cache on read.

### Engine

- **REQ-E-001**: The `Engine` struct MUST compose `DashMapCache` + `Box<dyn StorageBackend>`.
- **REQ-E-002**: The Engine MUST provide session CRUD: `create_session`, `get_session`, `list_sessions`, `update_session`, `delete_session`.
- **REQ-E-003**: Session listing MUST support filtering by `project` and pagination (`limit` + `offset`).
- **REQ-E-004**: The Engine MUST provide memory CRUD: `create_memory`, `get_memory`, `search_memories`, `update_memory`, `delete_memory`.
- **REQ-E-005**: Memory search MUST support filtering by `memory_type`, `tags`, `session_id`, `agent_id`, and full-text keyword search via a configurable field.
- **REQ-E-006**: The Engine MUST provide generic `store(cf, key, value)` and `get(cf, key)` for flexible key-value access.
- **REQ-E-007**: The Engine MUST expose `storage_size()` returning per-CF sizes.
- **REQ-E-008**: The Engine MUST expose `checkpoint()` for WAL flush.

### PyO3 Bridge

- **REQ-P-001**: A single `#[pyclass] Engine` struct MUST be exposed from Rust. All methods are `#[pymethods]`.
- **REQ-P-002**: All Python-facing types MUST be converted via serde JSON (Rust struct → JSON string → Python dict).
- **REQ-P-003**: The Python `core_bridge.py` wrapper MUST provide async methods using `asyncio.to_thread()`.
- **REQ-P-004**: A `ThreadPoolExecutor(max_workers=4)` MUST be used for bridge calls.
- **REQ-P-005**: Rust panics at the bridge boundary MUST be caught via `catch_unwind` and converted to `PyErr`.

### CLI

- **REQ-L-001**: The CLI MUST provide a `contexter` command with subcommands for diagnostics.
- **REQ-L-002**: `contexter status` MUST display data directory path, per-CF sizes, total entity counts, and cache hit ratio.
- **REQ-L-003**: `contexter session create|list|get|delete` MUST support session CRUD.
- **REQ-L-004**: `contexter memory create|search` MUST support memory creation and keyword search.

### Compression

- **REQ-Z-001**: Zstd wrapper MUST support configurable compression levels (1–22).
- **REQ-Z-002**: LZ4 wrapper MUST support standard LZ4 block mode.
- **REQ-Z-003**: Both wrappers MUST implement a shared `Compression` trait.

### Testing

- **REQ-TT-001**: Every Rust source file MUST have inline `#[cfg(test)] mod tests`.
- **REQ-TT-002**: Integration tests in `tests/` MUST mirror the `src/` module structure.
- **REQ-TT-003**: RocksDB integration tests MUST use a temporary directory (tempfile crate).
- **REQ-TT-004**: The test suite MUST cover: session CRUD, memory CRUD, cache hit/miss, RocksDB WAL recovery, key encoding correctness, compression round-trips, and PyO3 JSON round-trips.
- **REQ-TT-005**: `cargo clippy` MUST pass with no warnings.

### Configuration

- **REQ-CF-001**: The Engine MUST accept a `StorageConfig` struct with `path`, `engine` type, and cache settings.
- **REQ-CF-002**: Default data path is `~/.contexter/`.

### Constraints

- **CON-001**: No external database processes allowed. Everything must be in-process (RocksDB is embedded).
- **CON-002**: No network calls between Rust and Python (PyO3 direct calls only).
- **CON-003**: UUID v7 is mandatory for all primary keys.
- **CON-004**: All timestamps MUST be UTC.
- **CON-005**: The CLI tool MUST work without the Python API layer running.
- **CON-006**: All serde representations MUST use camelCase for JSON field names.

## 4. Interfaces & Data Contracts

### StorageBackend Trait (Rust)

```rust
pub trait StorageBackend: Send + Sync {
    // Session CRUD
    fn create_session(&self, session: NewSession) -> Result<Session>;
    fn get_session(&self, id: Uuid) -> Result<Option<Session>>;
    fn list_sessions(&self, filter: &SessionFilter) -> Result<Vec<Session>>;
    fn update_session(&self, id: Uuid, patch: &SessionPatch) -> Result<Session>;
    fn delete_session(&self, id: Uuid) -> Result<()>;
    fn count_sessions(&self, filter: &SessionFilter) -> Result<u64>;

    // Memory CRUD
    fn create_memory(&self, memory: NewMemory) -> Result<Memory>;
    fn get_memory(&self, id: Uuid) -> Result<Option<Memory>>;
    fn search_memories(&self, query: &MemorySearchQuery) -> Result<Vec<Memory>>;
    fn update_memory(&self, id: Uuid, patch: &MemoryPatch) -> Result<Memory>;
    fn delete_memory(&self, id: Uuid) -> Result<()>;
    fn count_memories(&self, filter: &MemoryFilter) -> Result<u64>;

    // Agent CRUD
    fn create_agent(&self, agent: NewAgent) -> Result<Agent>;
    fn get_agent(&self, id: Uuid) -> Result<Option<Agent>>;
    fn list_agents(&self, filter: &AgentFilter) -> Result<Vec<Agent>>;
    fn update_agent(&self, id: Uuid, patch: &AgentPatch) -> Result<Agent>;
    fn delete_agent(&self, id: Uuid) -> Result<()>;

    // Skill CRUD
    fn create_skill(&self, skill: NewSkill) -> Result<Skill>;
    fn get_skill(&self, id: Uuid) -> Result<Option<Skill>>;
    fn list_skills(&self, filter: &SkillFilter) -> Result<Vec<Skill>>;
    fn update_skill(&self, id: Uuid, patch: &SkillPatch) -> Result<Skill>;
    fn delete_skill(&self, id: Uuid) -> Result<()>;

    // Settings
    fn get_setting(&self, key: &str) -> Result<Option<String>>;
    fn set_setting(&self, key: &str, value: &str) -> Result<()>;

    // Audit
    fn append_audit_entry(&self, entry: &NewAuditEntry) -> Result<()>;
    fn query_audit_log(&self, filter: &AuditFilter) -> Result<Vec<AuditEntry>>;

    // Maintenance
    fn flush(&self) -> Result<()>;
    fn checkpoint(&self) -> Result<u64>;
    fn storage_size(&self) -> Result<StorageSize>;
}
```

### Engine (PyO3) — Python View

```python
# Type: dict with camelCase keys, matching JSON serialization
# All methods: async, wrapped via asyncio.to_thread()

class Engine:
    def __init__(self, config: dict): ...

    # Session
    async def create_session(self, data: dict) -> dict: ...
    async def get_session(self, id: str) -> dict | None: ...
    async def list_sessions(self, filter: dict) -> list[dict]: ...
    async def update_session(self, id: str, patch: dict) -> dict: ...
    async def delete_session(self, id: str) -> None: ...

    # Memory
    async def create_memory(self, data: dict) -> dict: ...
    async def get_memory(self, id: str) -> dict | None: ...
    async def search_memories(self, query: dict) -> dict: ...
    async def update_memory(self, id: str, patch: dict) -> dict: ...
    async def delete_memory(self, id: str) -> None: ...

    # Generic KV
    async def store(self, cf: str, key: str, value: str) -> None: ...
    async def get(self, cf: str, key: str) -> str | None: ...

    # Maintenance
    async def checkpoint(self) -> int: ...
    async def storage_size(self) -> dict: ...
    async def status(self) -> dict: ...
```

### Key Encoding Rules

| Entity | Key Pattern | CF | Example |
|---|---|---|---|
| Memory | `mem:{uuid}` | memory_items | `mem:01J123456789ABCDEF` |
| Session | `ses:{uuid}` | sessions | `ses:01J123456789ABCDEF` |
| Agent | `agt:{uuid}` | agents | `agt:01J123456789ABCDEF` |
| Skill | `skl:{uuid}` | skills | `skl:01J123456789ABCDEF` |
| Setting | `cfg:{key}` | sessions (or dedicated CF) | `cfg:storage.path` |
| Audit | `aud:{uuid}` | sessions (or dedicated CF) | `aud:01J123456789ABCDEF` |

### SessionFilter

```rust
pub struct SessionFilter {
    pub project: Option<String>,
    pub agent_id: Option<Uuid>,
    pub status: Option<SessionStatus>,
    pub limit: u64,
    pub offset: u64,
}
```

### MemorySearchQuery

```rust
pub struct MemorySearchQuery {
    pub keywords: Option<String>,      // full-text keyword search
    pub memory_type: Option<MemoryType>,
    pub tags: Option<Vec<String>>,
    pub session_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub project: Option<String>,
    pub limit: u64,
    pub offset: u64,
}
```

### StorageSize

```rust
pub struct StorageSize {
    pub per_cf: HashMap<String, u64>,  // CF name → bytes
    pub wal_size: u64,
    pub total: u64,
}
```

## 5. Acceptance Criteria

See `ACCEPTANCE.md` for the full Given/When/Then criteria.

## 6. Test Automation Strategy

- **Test Levels:** Unit (inline `#[cfg(test)]`), Integration (`tests/` directory mirroring `src/`)
- **Frameworks:** built-in `#[test]`, `cargo test`, `cargo clippy`
- **Test Data:** Sample entities generated in `tests/common/mod.rs`
- **Temp Dir:** `tempfile::TempDir` for RocksDB test instances
- **Coverage:** Every public function tested. No uncovered error paths.
- **CI:** `cargo test && cargo clippy -- -D warnings` must pass

## 7. Rationale & Context

- **RocksDB over SQLite:** RocksDB is an embedded LSM tree with native column family support, per-CF compression, and built-in WAL. No external process needed. Handles 10GB+ with stable latency. SQLite was rejected because it lacks native vector storage, column-level compression control, and its FTS5 is slower than Tantivy.
- **Synchronous StorageBackend:** RocksDB operations are inherently synchronous. Async wrapping at the Python layer via `asyncio.to_thread()` is the correct boundary.
- **UUID v7 over UUID v4:** Time-ordered UUIDs are sortable, enabling efficient range scans by creation time within RocksDB's ordered key structure.
- **DashMap + LRU:** DashMap provides lock-free concurrent reads. LRU eviction per entity type prevents one entity type from crowding out another.
- **JSON at PyO3 boundary:** Simple, debuggable, avoids complex PyO3 type mapping. Performance-critical paths can be optimized later.

## 8. Dependencies & External Integrations

### Rust Crates

| Crate | Purpose | Version Constraint |
|---|---|---|
| `rust-rocksdb` | RocksDB bindings | 0.22+ |
| `pyo3` | Python bridge | 0.21+ |
| `maturin` | Build tool for PyO3 | 1.5+ |
| `serde` / `serde_json` | Serialization | 1.x |
| `uuid` | UUID v7 generation | 1.x (with `v7` feature) |
| `chrono` | UTC timestamps | 0.4.x |
| `sha2` | SHA-256 hashing | 0.10.x |
| `dashmap` | Concurrent hashmap | 5.x |
| `tracing` | Structured logging | 0.1.x |
| `tempfile` | Test temp directories | 3.x |
| `thiserror` | Error type derivation | 1.x |
| `anyhow` | Error handling | 1.x |

### Python Dependencies

| Package | Purpose |
|---|---|
| `click` | CLI framework |
| `contexter-core` | Our own PyO3 wheel (built via maturin) |

### Build Tooling

| Tool | Purpose |
|---|---|
| `maturin` | Build PyO3 wheels from Rust |
| `cargo` | Rust build, test, clippy |

## 9. Examples & Edge Cases

See `EDGE_CASES.md` for the full edge case catalog.

## 10. Validation Criteria

- [ ] `cargo test` passes with all tests green
- [ ] `cargo clippy -- -D warnings` passes clean
- [ ] Python script can import `contexter_core` and create/list sessions
- [ ] CLI `contexter status` shows correct data directory and entity counts
- [ ] CRUD operations on sessions and memories round-trip correctly
- [ ] Cache MISS→L2 fallthrough works and populates cache
- [ ] WAL replay recovers unflushed writes after simulated crash
- [ ] Concurrent reads (2+ threads) do not deadlock
- [ ] Memory search by keywords returns correct results
- [ ] Per-CF compression settings are applied correctly
- [ ] All entity types (agent, skill) have working CRUD paths

## 11. Related Specifications / Further Reading

- [System Architecture](../2026-07-23-contexter-system-architecture.md)
- [Specification Hub](../2026-07-23-contexter-specification-hub.md)
- [Full Implementation Plan](../../plans/2026-07-23-contexter-implementation-plan.md)
