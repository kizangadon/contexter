# Contexter Phase 1 — Rust Core Foundation: Design Draft

> **Status:** `DRAFT — Pending Review` · **Version:** `v0.1.0-draft`
> **Feature:** 1 Architecture Approach · 0 Open Questions

---

## Navigation

- [Problem](#problem)
- [Options](#options)
- [System Design](#architecture)
- [Data Flow](#dataflow)
- [Questions](#questions)
- [Decisions](#decisions)
- [API](#api)
- [Scope](#scope)
- [AC](#ac)
- [Edge Cases](#edgecases)
- [Summary](#summary)

---

## Why This Feature Exists {#problem}

| The Pain | The Principle |
|---|---|
| Contexter needs a durable, in-process storage engine for AI agent memory, sessions, agent definitions, and skill catalogs. External databases add deployment complexity and network latency. SQLite lacks column-level compression control, native vector storage, and its FTS5 is slower than dedicated alternatives. No existing Rust library provides the multi-tier (cache → primary → vector → FTS → analytics) architecture needed. | "Embed everything, expose via PyO3, compose at the Engine level." One process, zero network hops between Rust and Python. RocksDB provides the LSM-tree foundation; DashMap adds a hot cache; the Engine struct composes all tiers into a single coherent API that the Python layer consumes as a native module. |

---

## Design Options {#options}

One architecture approach — this is infrastructure, not a feature with multiple UX paths. The decisions are technical:

### Option A (Selected) — RocksDB Multi-Tier with PyO3 Bridge

Layered storage: DashMap+LRU hot cache (L1) → RocksDB column families (L2) → with stubs for L3–L5. All exposed via a single `#[pyclass] Engine`.

### Option B — SQLite with Application-Level Sharding

Single SQLite database with separate tables per entity type. FTS5 for text search. Custom Rust HNSW vector index on the side.

**Rejected** because:
- No per-table compression control (affects telemetry CF efficiency)
- FTS5 slower than Tantivy for production-scale text search
- No column family isolation (one table's compaction affects others)
- Harder to evolve schema across 8+ entity types

### Option C — Pure Python with SQLAlchemy + pgvector

Python-native stack: SQLAlchemy ORM, PostgreSQL via asyncpg, pgvector for vectors, no Rust layer.

**Rejected** because:
- Requires external PostgreSQL server (deployment complexity)
- Network latency for every operation
- No hot cache layer — every read is a network round-trip
- pgvector ANN slower than in-process HNSW at 1M+ embeddings

---

## System Design {#architecture}

> **Status:** `Draft`

### Module Architecture

```
Python CLI / Wrapper
       │ asyncio.to_thread()
       ▼
┌──────────────────────────────────────────────────────────┐
│  Engine (#[pyclass])                                      │
│  ┌────────────────┐  ┌──────────────────────────────────┐ │
│  │ DashMapCache    │  │ Box<dyn StorageBackend>          │ │
│  │ (L1: Hot Cache) │  │ ┌────────────────────────────┐  │ │
│  │ write-through   │  │ │ RocksDbBackend (L2)        │  │ │
│  │ write-around    │  │ │ ┌──────────┐ ┌──────────┐ │  │ │
│  └────────────────┘  │ │ │mem CF    │ │ses CF    │ │  │ │
│                      │ │ │(Zstd,64M)│ │(Zstd,32M)│ │  │ │
│  + stubs for L3-L5   │ │ ├──────────┤ ├──────────┤ │  │ │
│  (vector, fts,       │ │ │agt CF    │ │skl CF    │ │  │ │
│   analytics)          │ │ │(LZ4,16M) │ │(LZ4,16M) │ │  │ │
│                      │ │ ├──────────┤ ├──────────┤ │  │ │
│                      │ │ │eff CF    │ │tel CF    │ │  │ │
│                      │ │ │(LZ4,8M)  │ │(LZ4,4M)  │ │  │ │
│                      │ │ ├──────────┤ ├──────────┤ │  │ │
│                      │ │ │cfl CF    │ │idx CF    │ │  │ │
│                      │ │ │(Zstd,8M) │ │(LZ4,4M)  │ │  │ │
│                      │ │ └──────────┘ └──────────┘ │  │ │
│                      │ └────────────────────────────┘  │ │
│                      └──────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```

### Data Model

**Session** `[L2: sessions CF — Zstd, key: ses:{uuid}]`
```
id                   UUID v7
project              str
agent_id             UUID v7
status               enum(active, completed, error)
turn_count           u32
duration_ms          u64
metadata             json
created_at           datetime (UTC)
last_active          datetime (UTC)
```

**Memory** `[L2: memory_items CF — Zstd, key: mem:{uuid}]`
```
id                   UUID v7
session_id           UUID v7
agent_id             UUID v7
type                 enum(fact, preference, procedure, context, episode)
content              text
embedding            f32[] [stub — populated in Phase 2]
tags                 str[]
version              u32
created_at           datetime (UTC)
updated_at           datetime (UTC)
```

**Agent** `[L2: agents CF — LZ4, key: agt:{uuid}]`
```
id                   UUID v7
name                 str
type                 str
description          str
capabilities         str[]
status               enum(active, inactive)
config               json
version              u32
created_at           datetime (UTC)
updated_at           datetime (UTC)
```

**Skill** `[L2: skills CF — LZ4, key: skl:{uuid}]`
```
id                   UUID v7
name                 str
description          str
category             str
version              u32
file_path            str
created_at           datetime (UTC)
updated_at           datetime (UTC)
```

### API Layer (Rust → Python via PyO3)

```
Python Script / CLI
       │  Engine.create_session(data)  → serde JSON → Rust struct
       │  Engine.get_session(id)       → Rust struct → serde JSON → Python dict
       ▼
┌──────────────────────────────────────────────────────────┐
│  bridge.rs                                                │
│  #[pyclass] Engine {                                       │
│      #[pymethod] fn create_session(&self, ...) -> PyResult │
│      #[pymethod] fn get_session(&self, ...) -> PyResult    │
│      #[pymethod] fn list_sessions(&self, ...) -> PyResult  │
│      ...                                                    │
│  }                                                          │
└──────────────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────────────┐
│  Engine (Rust)                                            │
│  engine::session::create_session()                         │
│       → cache.invalidate("ses:{id}")                       │
│       → storage.create_session(new_session)                 │
│       → bridge.serialize_to_json(result)                    │
└──────────────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────────────┐
│  RocksDbBackend                                           │
│  → rocksdb.put_cf(sessions_cf, "ses:{uuid}", bytes)       │
│  → rocksdb.flush_wal(true)                                 │
└──────────────────────────────────────────────────────────┘
```

---

## Data Flow {#dataflow}

### 1. Engine Initialization

```
Python: engine = Engine({"path": "~/.contexter/"})
  → Rust: Engine::new(config)
    → RocksDB opens at path with 8 column families
    → DashMapCache::new(10000) — per-type capacity
    → Engine { cache, storage } returned to Python
```

### 2. Create Session

```
Python: session = await engine.create_session({"project": "test", ...})
  → asyncio.to_thread(self._rust.create_session, data)
    → serde_json::from_str(data) → NewSession
    → cache.write_through("ses:{uuid}", session)  // cache + storage
    → RocksDbBackend::create_session
        → uuid_v7() for id, chrono::Utc::now() for timestamps
        → rocksdb.put_cf(sessions_cf, key, serde_json::to_vec(&session))
        → rocksdb.flush_wal(true)  // durability
    → Engine::cache.store("ses:{uuid}", session)  // populate cache
    → serde_json::to_string(&session) → Python dict
```

### 3. Read Session (Cache Hit Path)

```
Python: session = await engine.get_session(id)
  → asyncio.to_thread(self._rust.get_session, id)
    → cache.get("ses:{id}") → Some(session)  // HIT
    → telemetry.record_cache_hit("session")
    → return session
  ← ~50ns latency
```

### 4. Read Session (Cache Miss Path)

```
Python: session = await engine.get_session(id)
  → asyncio.to_thread(self._rust.get_session, id)
    → cache.get("ses:{id}") → None  // MISS
    → telemetry.record_cache_miss("session")
    → RocksDbBackend::get_session(id)
        → rocksdb.get_cf(sessions_cf, "ses:{id}")
        → serde_json::from_slice(bytes) → Session
    → cache.store("ses:{id}", session)  // populate cache
    → return session
  ← ~50µs-5ms latency (RocksDB read)
```

### 5. Search Memories (Keyword)

```
Python: results = await engine.search_memories({"keywords": "auth", ...})
  → asyncio.to_thread(self._rust.search_memories, query)
    → no L4 Tantivy yet (Phase 2)
    → RocksDbBackend::search_memories
        → iterate memory_items CF
        → filter by memory_type, tags, session_id, agent_id
        → for keyword search: simple substring match on content
        → score by: exact match > prefix match > substring match
        → sort by score descending
        → apply limit + offset
    → return scored results
```

---

## Open Questions {#questions}

| ID | Question | Status |
|---|---|---|
| OQ-001 | Should keyword search in Phase 1 use a simple substring scan (iterate all memories, match in Rust) or defer all text search to Tantivy (Phase 2)? The spec says "keyword search via a configurable field" — substring scan is simpler for Phase 1 but won't scale. | ✅ **Resolved** — substring scan with scoring, documented as replaced by Tantivy in Phase 2 |
| OQ-002 | Should the generic `store(cf, key, value)` accept arbitrary CF names or only the 8 defined ones? Accepting arbitrary could lead to typos creating unexpected CFs. | ✅ **Resolved** — validate against allowed CF list, return error for unknown |
| OQ-003 | Should `memory_items` have a dedicated secondary index CF for by-session and by-agent lookups, or should those be prefix scans within the same CF? | ✅ **Resolved** — prefix scans within memory_items (UUID v7 prefix included in key) |
| OQ-004 | Should `delete_session` cascade-delete associated memories? The spec says no cascade (memories survive). | ✅ **Resolved** — no cascade. Memories are orphaned until explicitly deleted. |

---

## Decision Log {#decisions}

| Date | ID | Description | Rationale |
|---|---|---|---|
| 2026-07-23 | CON-001 | RocksDB multi-tier over SQLite or Python-native | Embedded LSM tree, per-CF compression, no external process, built-in WAL |
| 2026-07-23 | CON-002 | DashMap + LRU for hot cache | Lock-free concurrent reads, ~50ns latency, per-type LRU prevents cross-type eviction |
| 2026-07-23 | CON-003 | UUID v7 for all primary keys | Time-ordered, sortable, enables range scans by creation time |
| 2026-07-23 | CON-004 | Synchronous StorageBackend trait | RocksDB ops are sync; async wrapping at Python layer is correct boundary |
| 2026-07-23 | CON-005 | JSON at PyO3 boundary | Simple, debuggable, avoids complex PyO3 type mapping. Optimize later if needed |
| 2026-07-23 | CON-006 | Keyword search via substring scan in Phase 1 | Workable for small datasets. Replaced by Tantivy BM25 in Phase 2 |
| 2026-07-23 | CON-007 | No cascade delete from session to memories | Memories are durable records. Orphaned memories are explicitly cleaned up or reparented |
| 2026-07-23 | CON-008 | CF name validation on generic store/get | Prevents silent typos from creating unexpected column families |

---

## API Contract {#api}

> **Status:** `Draft`

### Rust `StorageBackend` Trait (Key Signatures)

```rust
pub trait StorageBackend: Send + Sync {
    fn create_session(&self, session: NewSession) -> Result<Session>;
    fn get_session(&self, id: Uuid) -> Result<Option<Session>>;
    fn list_sessions(&self, filter: &SessionFilter) -> Result<Vec<Session>>;
    fn update_session(&self, id: Uuid, patch: &SessionPatch) -> Result<Session>;
    fn delete_session(&self, id: Uuid) -> Result<()>;

    fn create_memory(&self, memory: NewMemory) -> Result<Memory>;
    fn get_memory(&self, id: Uuid) -> Result<Option<Memory>>;
    fn search_memories(&self, query: &MemorySearchQuery) -> Result<Vec<Memory>>;
    fn update_memory(&self, id: Uuid, patch: &MemoryPatch) -> Result<Memory>;
    fn delete_memory(&self, id: Uuid) -> Result<()>;

    fn create_agent(&self, agent: NewAgent) -> Result<Agent>;
    fn get_agent(&self, id: Uuid) -> Result<Option<Agent>>;
    fn list_agents(&self, filter: &AgentFilter) -> Result<Vec<Agent>>;
    fn update_agent(&self, id: Uuid, patch: &AgentPatch) -> Result<Agent>;
    fn delete_agent(&self, id: Uuid) -> Result<()>;

    // ... skills, settings, audit, maintenance
}
```

### Python `Engine` API (Async)

```python
# All methods return dicts with camelCase keys

# Session
async def create_session(self, data: dict) -> dict
async def get_session(self, id: str) -> dict | None
async def list_sessions(self, filter: dict) -> list[dict]
async def update_session(self, id: str, patch: dict) -> dict
async def delete_session(self, id: str) -> None

# Memory
async def create_memory(self, data: dict) -> dict
async def get_memory(self, id: str) -> dict | None
async def search_memories(self, query: dict) -> dict  # {results: [...], total: int}
async def update_memory(self, id: str, patch: dict) -> dict
async def delete_memory(self, id: str) -> None

# Agent
async def create_agent(self, data: dict) -> dict
async def get_agent(self, id: str) -> dict | None
async def list_agents(self, filter: dict) -> list[dict]

# Skill
async def create_skill(self, data: dict) -> dict
async def get_skill(self, id: str) -> dict | None
async def list_skills(self, filter: dict) -> list[dict]

# Generic KV
async def store(self, cf: str, key: str, value: str) -> None
async def get(self, cf: str, key: str) -> str | None

# Maintenance
async def checkpoint(self) -> int
async def storage_size(self) -> dict
async def status(self) -> dict  # health, counts, sizes, cache ratio
```

### CLI Commands

```
contexter status                              → engine health + stats
contexter session create --project <p> ...    → creates session, prints id
contexter session list [--project <p>]        → lists sessions as JSON
contexter session get <id>                     → session details as JSON
contexter session delete <id>                  → deletes session
contexter memory create --session <id> ...    → creates memory, prints id
contexter memory search --keywords <k>         → search results as JSON
```

### Field Specification — Session

| Field | Type | Required | Default | Constraints |
|---|---|---|---|---|
| `project` | string | required | — | non-empty |
| `agent_id` | string (UUID) | required | — | valid UUID v7 |
| `status` | string | optional | "active" | one of: active, completed, error |
| `turn_count` | integer | optional | 0 | ≥ 0 |
| `duration_ms` | integer | optional | 0 | ≥ 0 |
| `metadata` | object | optional | {} | valid JSON |

### Field Specification — Memory

| Field | Type | Required | Default | Constraints |
|---|---|---|---|---|
| `session_id` | string (UUID) | required | — | valid UUID v7 |
| `agent_id` | string (UUID) | required | — | valid UUID v7 |
| `type` | string | required | — | one of: fact, preference, procedure, context, episode |
| `content` | string | required | — | non-empty |
| `tags` | string[] | optional | [] | max 50 tags, each ≤ 100 chars |

---

## Out of Scope {#scope}

| # | Item | Rationale |
|---|---|---|
| 01 | Vector embeddings (L3) | Phase 2. HNSW vector index is separate from CRUD storage. |
| 02 | Full-text search with BM25 (L4) | Phase 2. Tantivy replaces substring scan. |
| 03 | Analytics engine (L5) | Phase 2. DuckDB queries on aggregated data. |
| 04 | FastAPI REST server | Phase 3. All access goes through PyO3 bridge directly. |
| 05 | FastMCP server | Phase 3. MCP tools are Python-layer concerns. |
| 06 | React UI | Phase 4. No browser interface in Phase 1. |
| 07 | Auto-versioning file watcher | Phase 4. `notify` crate integration deferred. |
| 08 | Multi-user auth | Future. Single-user default in all phases. |
| 09 | Export / report generation | Phase 4. Requires analytics engine first. |
| 10 | Conflict resolution UI | Phase 4. CRDT conflict records stored but not surfaced. |

---

## Acceptance Criteria {#ac}

> **Status:** 24 Pending

| ID | Description | Status |
|---|---|---|
| AC-001 | RocksDB engine initializes with all 8 column families and correct per-CF compression | 🔶 Pending |
| AC-002 | Session is created and retrieved (round-trip) | 🔶 Pending |
| AC-003 | Session is listed with project filter and pagination | 🔶 Pending |
| AC-004 | Session is updated and changes persist | 🔶 Pending |
| AC-005 | Session is deleted (subsequent get returns None) | 🔶 Pending |
| AC-006 | Memory is created with type and tags, version starts at 1 | 🔶 Pending |
| AC-007 | Memory is searched by keyword | 🔶 Pending |
| AC-008 | Memory is searched with type + tag filters | 🔶 Pending |
| AC-009 | Memory version increments on update | 🔶 Pending |
| AC-010 | Memory is deleted (subsequent get returns None) | 🔶 Pending |
| AC-011 | Agent and skill CRUD round-trips correctly | 🔶 Pending |
| AC-012 | Generic store/get works cross-CF with CF isolation | 🔶 Pending |
| AC-013 | Cache hit returns data without RocksDB read | 🔶 Pending |
| AC-014 | Cache miss falls through to RocksDB and populates cache | 🔶 Pending |
| AC-015 | CLI status shows data dir, per-CF sizes, entity counts, cache ratio | 🔶 Pending |
| AC-016 | CLI session CRUD works end-to-end | 🔶 Pending |
| AC-017 | PyO3 bridge round-trips session creation from Python | 🔶 Pending |
| AC-018 | Zstd and LZ4 compression round-trips correctly | 🔶 Pending |
| AC-019 | WAL checkpoint flushes and reduces WAL size | 🔶 Pending |
| AC-020 | Storage size reports per-CF breakdown | 🔶 Pending |
| AC-101 | Invalid UUID on create returns error | 🔶 Pending |
| AC-102 | Get non-existent entity returns None | 🔶 Pending |
| AC-103 | Delete non-existent entity returns Ok (idempotent) | 🔶 Pending |
| AC-104 | Update non-existent entity returns error | 🔶 Pending |
| AC-105 | Read-only storage path returns error on init | 🔶 Pending |
| AC-106 | Concurrent reads from 4 threads succeed | 🔶 Pending |
| AC-107 | Large content (1MB) memory round-trips correctly | 🔶 Pending |
| AC-108 | Engine works with empty/fresh database | 🔶 Pending |
| AC-201 | Cache read latency under 100µs | 🔶 Pending |
| AC-202 | RocksDB write latency under 5ms | 🔶 Pending |
| AC-203 | All cargo tests pass, clippy clean | 🔶 Pending |

---

## Edge Cases {#edgecases}

> **Status:** 47 Identified across 8 categories

| ID | Scenario | Expected Behavior | Priority |
|---|---|---|---|
| E-001 | Data directory doesn't exist | RocksDB creates it (create_if_missing=true) | High |
| E-002 | Empty data directory (fresh install) | Engine opens, CFs created, all counts zero | High |
| E-005 | Data directory not writable | Error: "path not writable" | High |
| E-010 | MANIFEST corrupted | RocksDB corruption error, suggest WAL recovery | High |
| E-101 | Session listing with no filter | Returns all sessions paginated (limit=100) | High |
| E-104 | Listing offset beyond total | Returns empty list | Medium |
| E-106 | Update with empty patch | No-op, returns session unchanged | Medium |
| E-108 | Create session with missing required field | Validation error | High |
| E-201 | Memory with empty content string | Created, searchable only via type/tag filters | Medium |
| E-204 | Memory search with empty query | Returns all memories (limit=100) | High |
| E-210 | Query memories for session with none | Empty results | High |
| E-301 | Cache hit on recently written entity | Returns from cache, no RocksDB read | High |
| E-302 | Cache miss after LRU eviction | Falls through to RocksDB, re-caches | High |
| E-303 | Cache invalidation on update | Stale entry removed | High |
| E-307 | One entity type fills entire cache | Per-type LRU prevents cross-type eviction | High |
| E-401 | Disk full during write | Error returned, WAL ensures consistency | High |
| E-503 | Python calls with wrong arg types | PyO3 TypeError before Rust code | High |
| E-701 | Compress empty byte slice | Returns empty/minimal bytes | Medium |
| E-703 | Decompress corrupted data | Error: "Decompression failed" | High |

*Full catalog of 47 edge cases in `EDGE_CASES.md`*

---

## Design Draft Summary {#summary}

| Metric | Count |
|---|---|
| Acceptance Criteria | 31 |
| Edge Cases | 47 (across 8 categories) |
| Design Options | 3 (1 selected, 2 rejected) |
| Open Questions | 4 (all resolved) |
| Decision Log Entries | 8 |
| Rust Modules | 17 (lib.rs + 16 modules) |
| Python Files | 2 (bridge wrapper + CLI) |
| Rust Crates | 14+ |

This draft covers Phase 1 of Contexter — the foundational Rust core engine with RocksDB multi-tier storage, PyO3 bridge, and CLI diagnostics. All 4 open questions have been resolved inline.

---

**Generated · 2026-07-23 · Contexter Phase 1 — Rust Core Foundation Design Draft · v0.1.0-draft**

<!-- LOCKED: Template finalized on 2026-07-23 -->
