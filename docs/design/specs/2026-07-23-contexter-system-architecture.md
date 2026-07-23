# Contexter — System Architecture

**Date:** 2026-07-23
**Status:** Draft
**Parent Hub:** [2026-07-23-contexter-specification-hub.md](2026-07-23-contexter-specification-hub.md)
**UI Design:** [2026-07-23-contexter-ui-design.md](2026-07-23-contexter-ui-design.md)

---

## 1. Overview

Contexter is a RAG-like memory, agent, skill, and session management platform for AI coding agents. It replaces the rekal system entirely, absorbing all memory, endpoint, and session functionality.

The system is a **modular monolith** — three layers in a single process, communicating through well-defined internal interfaces. The PyO3 bridge allows the Python management layer to call into the Rust core without serialization overhead, keeping everything in-process.

### High-Level Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                    React SPA (contexter-web)                      │
│                   (port 5173 dev / served prod)                    │
└────────────────────────────┬─────────────────────────────────────┘
                             │ HTTP (REST JSON, SSE for notifications)
                             ▼
┌──────────────────────────────────────────────────────────────────┐
│               Python Layer (contexter-server)                      │
│  ┌─────────────┐  ┌─────────────┐  ┌───────────────────────────┐ │
│  │ FastAPI      │  │ FastMCP     │  │ Services:                  │ │
│  │ (REST API)   │  │ (MCP Server)│  │  session, memory, agent,   │ │
│  │ port 8000    │  │ port 8001   │  │  skill, analytics, export, │ │
│  └─────────────┘  └─────────────┘  │  audit, notifications,     │ │
│                                     │  file_watcher, correlation  │ │
│                                     └───────────┬─────────────────┘ │
│                                                 │ PyO3 call          │
│  ┌──────────────────────────────────────────────┴──────────────────┐ │
│  │ core_bridge.py          (wraps Rust module, sync + async)        │ │
│  └──────────────────────────────────────────────┬──────────────────┘ │
└─────────────────────────────────────────────────┼────────────────────┘
                                                   │
                                                   ▼
┌──────────────────────────────────────────────────────────────────────┐
│                    Rust Core (contexter-core)                         │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │              Tiered Storage Architecture (L1–L5)                │ │
│  │                                                                  │ │
│  │  L1: Hot Cache     ───  DashMap + LRU (~50ns)                   │ │
│  │       write-through (memory writes commit sync to L2)            │ │
│  │       write-around (vector writes skip cache, invalidate on L3   │ │
│  │                     update)                                       │ │
│  │                                                                  │ │
│  │  L2: Primary Store  ───  RocksDB (8+ column families)            │ │
│  │       per-CF: Zstd (hot CFs) / LZ4 (speed-sensitive CFs)        │ │
│  │       built-in WAL with fsync + periodic checkpoint              │ │
│  │                                                                  │ │
│  │  L3: Vector Index   ───  HNSW (instant-distance / voyager)      │ │
│  │       in-memory graph, persisted as binary snapshot              │ │
│  │                                                                  │ │
│  │  L4: Full-Text Search ───  Tantivy (Lucene-class BM25)          │ │
│  │       indexed on write, queried on hybrid search                 │ │
│  │                                                                  │ │
│  │  L5: Analytics Engine ───  DuckDB (columnar, in-process)        │ │
│  │       on-demand sync from RocksDB, no replication lag            │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│  ┌──────────┐ ┌───────────┐ ┌──────────┐ ┌────────────┐ ┌────────┐ │
│  │ CRDT      │ │ Versioning│ │ Compress │ │ Self-Observ│ │ PyO3   │ │
│  │ LWW-Reg   │ │ Content-  │ │ Zstd/LZ4 │ │ cache hits, │ │ Bridge │ │
│  │           │ │ Addr Store│ │ per-CF   │ │ lat percent│ │ #[pyfn]│ │
│  └──────────┘ └───────────┘ └──────────┘ └────────────┘ └────────┘ │
└──────────────────────────────────────────────────────────────────────┘
```

### 1.1 Key Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| **Monolith vs microservices** | Modular monolith | Single deployable, zero network calls between Rust and Python. Easy to split later if needed. |
| **Rust ↔ Python bridge** | PyO3 (maturin) | Native Rust bindings exposed as a Python module. Zero serialization overhead for internal calls. |
| **Primary storage** | RocksDB + column families | Embedded, high-performance LSM tree. No external DB process. Handles 10GB+ with stable latency. WAL built-in. |
| **Hot cache** | DashMap + LRU | ~50ns read path for frequently accessed items. Decouples read throughput from RocksDB's LSM amplification. |
| **Vector index** | Rust-native HNSW | In-process, no external vector DB dependency. Persisted binary snapshot. Swappable for pgvector via plugin trait. |
| **Full-text search** | Tantivy | Lucene-class BM25, faster than SQLite FTS5, native Rust, supports incremental indexing. |
| **Analytics** | DuckDB | Columnar engine for analytical queries. In-process, no separate deployment. Sync from RocksDB on demand. |
| **Compression** | Zstd (hot CFs) + LZ4 (speed CFs) | Per-column-family strategy. Zstd for memory/session content (high ratio). LZ4 for telemetry (speed priority). |
| **API protocol** | REST + MCP | REST for the React UI and external integrations. MCP for AI agent tool access. Both share Python orchestration logic. |
| **Serialization** | Pydantic (Python) → serde (Rust) | Pydantic models validate and serialize in Python layer. PyO3 bridge converts to Rust's serde types at boundary. |
| **Conflict resolution** | CRDT LWW-Register | Last-Writer-Wins for concurrent memory edits. Timestamp-based merge. |
| **Versioning** | Content-addressed (SHA-256) | Hash → blob mapping. Reference-counted GC. Deduplication by content identity. |
| **Plugin storage** | `StorageBackend` trait | RocksDB is default. Trait enables future PostgreSQL/pgvector backends without changing orchestration code. |

---

## 2. Technology Stack

### 2.1 Rust Core (`contexter-core`)

| Component | Crate / Approach | Purpose |
|---|---|---|
| **L1: Hot Cache** | `dashmap` + custom LRU | Concurrent hashmap + LRU eviction. Write-through for memory/session reads. |
| **L2: Primary Store** | `rust-rocksdb` | RocksDB with 8+ column families. Built-in WAL, compression per CF, backup. |
| **L3: Vector Index** | `instant-distance` or `voyager` | HNSW graph for ANN search. Cosine similarity. Persisted binary snapshot. |
| **L4: Full-Text Search** | `tantivy` | BM25 index, incremental writes, query parser with field boosting. |
| **L5: Analytics Engine** | `duckdb` (via `rust-duckdb`) | Columnar SQL engine. On-demand sync query to pull data from RocksDB. |
| **Compression** | `zstd` + `lz4` | Zstd for content-heavy column families. LZ4 for speed-sensitive telemetry/index CFs. |
| **CRDT** | Custom `LWW-Register` | Last-Writer-Wins register. Same-type concurrent edit resolution by timestamp. |
| **Versioning** | Custom `ContentAddressedStore` | SHA-256 hashing via `sha2` crate. Maps hash → blob. Reference-counted GC. |
| **Hashing** | `sha2` | SHA-256 for content addressing and integrity checks. |
| **Diff** | `similar` | Line-level diff calculation for version comparison (additions, deletions, context). |
| **File watching** | `notify` | `inotify` (Linux) / `kqueue` (macOS) for auto-versioning. |
| **Serialization** | `serde` + `serde_json` | All Rust types `Serialize`/`Deserialize`. JSON wire format at PyO3 boundary. |
| **PyO3 bridge** | `pyo3` + `maturin` | `#[pyclass]`, `#[pyfunction]` declarations. Exposed as `contexter_core` Python package. |
| **Logging** | `tracing` | Structured logging. Hooks into Python's `structlog` via bridge. |
| **UUID** | `uuid` | UUID v7 (time-ordered) for primary keys. Sortable, cluster-friendly. |
| **Date/time** | `chrono` | UTC timestamps throughout. |

### 2.2 Python Layer (`contexter-server`)

| Component | Library | Purpose |
|---|---|---|
| REST API | `fastapi` + `uvicorn` | HTTP API on port 8000. Auto-generates OpenAPI spec. |
| MCP Server | `fastmcp` (MCP SDK) | MCP protocol server on port 8001. Exposes tools, resources, prompts. |
| Auth | `fastapi-users` + `jose` | JWT-based auth for REST API. MCP uses bearer token auth. |
| Validation | `pydantic-v2` | All request/response models. Single schema source of truth. |
| Background tasks | `arq` or `asyncio` | Export jobs, report generation, scheduled tasks, GC. |
| Notifications | Custom + `sse-starlette` | SSE for real-time notifications. Optional webhook dispatch. |
| Async | `anyio` / `asyncio` | Async throughout. Rust calls via PyO3 use `asyncio.to_thread`. |
| Logging | `structlog` | Structured JSON logging. Bridges to Rust `tracing`. |

### 2.3 React UI (`contexter-web`)

| Tool | Purpose |
|---|---|
| React 19 + TypeScript | UI framework |
| Tailwind CSS v4 | Styling (per V2-DEEP design system) |
| React Query (TanStack Query) | Server state, caching, optimistic updates |
| React Router v7 | Client-side routing |
| Recharts / visx | Charts and visualizations |
| Monaco editor | JSON editors in API Playground, settings |
| Framer Motion | Animations and transitions |

---

## 3. Tiered Storage Architecture (L1–L5)

The storage system is organized into five tiers. Each tier serves a distinct purpose and has a different performance profile. Data flows across tiers through explicit sync/invalidation paths — there is no automatic or transparent propagation.

```
Request Path (Read):
  Client → Python Service → L1 (DashMap+LRU) → [miss] → L2 (RocksDB) → [needs vector] → L3 (HNSW)
                                                                        ↕
                                                                     L4 (Tantivy)  ← hybrid search
                                                                        ↕
                                                                     L5 (DuckDB)   ← analytical queries

Write Path:
  Client → Python Service → L2 (RocksDB WAL → SST) → invalidate L1
                                                       → update L3 (HNSW insert)
                                                       → index L4 (Tantivy write)
                                                       → [async] L5 (DuckDB sync)
```

### 3.1 L1: Hot Cache (DashMap + LRU)

**Purpose:** Sub-microsecond read path for frequently accessed entities.

| Property | Value |
|---|---|
| Data structure | `DashMap<Key, Arc<Entry>>` + LRU eviction list |
| Typical latency | ~50ns (hash lookup) |
| Eviction policy | LRU with per-entity-type max capacity (configurable per CF) |
| Write policy | **Write-through** (memory/session/agent/skill writes → cache + RocksDB synchronously) |
|  | **Write-around** (vector writes bypass cache, invalidate stale entries on L3 update) |
| Invalidation | On L2 write, cache entry for key is removed (lazy re-population on next read) |
| Capacity | Default: 10,000 entries per entity type. Configurable in Settings > Storage. |
| Cold start | Empty on startup. Warmed by read traffic. |

Cache entries are keyed by the same string keys used in L2 (e.g., `"mem:uuid"`, `"ses:uuid"`). The LRU list tracks access order independently per entity type to prevent one type from crowding out others.

### 3.2 L2: Primary Store (RocksDB + Column Families)

**Purpose:** Durable, persistent storage for all entities and relationships.

#### Column Families

Each column family (CF) is a separate LSM tree within RocksDB, with its own compression, write buffer, and cache settings.

| CF Name | Content | Key Pattern | Compression | Write Buffer | Notes |
|---|---|---|---|---|---|
| `memory_items` | Memory entries + metadata | `mem:{uuid_v7}` | Zstd (level 3) | 64MB | Primary memory body. Largest CF. |
| `sessions` | Session state + checkpoints | `ses:{uuid_v7}` | Zstd (level 3) | 32MB | |
| `agents` | Agent definitions | `agt:{uuid_v7}` | LZ4 | 16MB | Low write volume, speed on read. |
| `skills` | Skill catalog entries | `skl:{uuid_v7}` | LZ4 | 16MB | |
| `efficiency_map` | Aggregated efficiency scores | `eff:{scope}:{period}` | LZ4 | 8MB | Computed once per session close. |
| `telemetry` | Observability events | `tel:{timestamp}:{id}` | LZ4 | 4MB | Very high write throughput. Speed > ratio. |
| `conflicts` | CRDT conflict records | `cfl:{entity_id}:{timestamp}` | Zstd (level 1) | 8MB | Low volume, needs durability. |
| `index_state` | Index metadata + checkpoints | `idx:{name}` | LZ4 | 4MB | Point lookups only. |

Additional CFs may be added during implementation:
- `settings` — key-value configuration pairs
- `notifications` — in-app notification records
- `audit_log` — audit trail entries (LZ4, high write volume)
- `feedback` — bug reports and feature suggestions
- `file_versions` — content-addressed version metadata
- `correlation` — pre-computed correlation results

#### Key Encoding

Keys are variable-length byte strings. The prefix determines the CF routing. All UUIDs are UUID v7 (time-ordered, sortable):

```
{entity_prefix}:{uuid_v7_or_other_id}[:{sub_key}]

Examples:
  mem:01J123456789ABCDEF   → memory_items CF, memory entity
  ses:01J123456789ABCDEF   → sessions CF, session entity  
  agt:01J123456789ABCDEF   → agents CF, agent entity
  skl:01J123456789ABCDEF   → skills CF, skill entity
  eff:mem:2026-07          → efficiency_map CF, memory efficiency for July 2026
  tel:20260723T143021Z:xxx → telemetry CF, event at timestamp
  cfl:mem:01J123456789ABC  → conflicts CF, conflict for memory
  idx:hnsw_snapshot        → index_state CF, HNSW snapshot path
```

#### WAL (Write-Ahead Log)

RocksDB's built-in WAL is used — not a custom component.

| Property | Setting |
|---|---|
| Mode | `wal::WriteOptions::set_sync(true)` — fsync on every write (default) |
| Recovery | Automatic on DB open. Replays unflushed WAL entries. |
| Checkpoint | Periodic `DB::flush_wal(true)`. Configurable interval (default: 60s). |
| Size limit | `max_total_wal_size` = 256MB. Beyond that, oldest WALs are deleted after flush. |
| Failover | On corruption, WAL replay from last successful checkpoint. Read-only mode if replay fails. |

#### Secondary Indices (within CF)

Within each column family, secondary lookups are supported via key prefixes:
- **By project:** Prefix-scan `ses:{project}:` within `sessions` CF
- **By agent:** Prefix-scan `mem:{agent_id}:` within `memory_items` CF (or dedicated index CF)
- **By time range:** UUID v7 is time-ordered, so `ses:{start_uuid}` to `ses:{end_uuid}` range scan works

For complex multi-attribute queries, the L4 Tantivy or L5 DuckDB tiers are preferred.

### 3.3 L3: Vector Index (HNSW)

**Purpose:** Approximate nearest neighbor search for semantic memory retrieval.

| Property | Value |
|---|---|
| Algorithm | Hierarchical Navigable Small World (HNSW) |
| Parameters | `M = 16`, `ef_construction = 200`, `ef_search = 50` |
| Distance | Cosine similarity (configurable to Euclidean/Dot) |
| Dimensions | 384 (default, configurable per embedding model) |
| Storage | In-memory graph + binary snapshot on disk |
| Snapshot path | `~/.contexter/vector_index.bin` |
| Auto-snapshot | Every 1,000 mutations + on graceful shutdown |

**Snapshot format:**
```
[4 bytes]  magic number (0x484E5357 = "HNSW")
[4 bytes]  version
[4 bytes]  dimension count (u32)
[4 bytes]  element count (u32)
[8 bytes]  M parameter
[8 bytes]  ef_construction parameter
[...]      graph adjacency list (packed)
[...]      embedding vectors (f32 × dim × count)
```

**Recovery:** On startup, the snapshot is loaded into memory. If the memory count in L2 doesn't match the index entry count, a full rebuild is triggered by iterating all `memory_items` with embeddings and re-inserting.

### 3.4 L4: Full-Text Search (Tantivy)

**Purpose:** BM25 keyword search across memory/session/agent/skill content.

| Property | Value |
|---|---|
| Engine | Tantivy (Lucene-class inverted index) |
| Index directory | `~/.contexter/tantivy_index/` |
| Schema | Per-entity-type indexed fields (content, title, tags, metadata) with field-level boosting |
| Indexing | Incremental — new documents added on L2 write |
| Merging | Automatic segment merging (Tantivy default) |

**Search query flow:**
1. Parse user query into Tantivy query AST (supports phrase, fuzzy, boolean)
2. Apply field boosts: `content:1.0`, `title:2.0`, `tags:1.5`
3. Execute search, returning top-N scored results with BM25 scores
4. Results are intersected with L3 HNSW results in hybrid search

### 3.5 L5: Analytics Engine (DuckDB)

**Purpose:** Columnar SQL queries for analytics, reporting, and correlation computation.

| Property | Value |
|---|---|
| Engine | DuckDB (in-process, columnar) |
| Data source | On-demand sync queries via RocksDB iterator → DuckDB in-memory tables |
| Persistence | None (ephemeral, rebuilt on each analytics request, or cached for configurable TTL) |
| Retention | Raw telemetry data in L2 `telemetry` CF. DuckDB queries aggregate from L2. |

**Sync mechanism:**
1. Analytics request arrives (e.g., "cost by model this month")
2. Python service calls `analytics_engine.query(sql, params)`
3. DuckDB executes attached SQL, which uses DuckDB's ability to read from arbitrary data sources
4. For large aggregations, data is iterated from RocksDB, materialized into a DuckDB in-memory table, then queried

**Alternative (simpler):** For Phase 1, analytics queries can bypass DuckDB entirely and aggregate directly in Python from RocksDB iterators. DuckDB is introduced in Phase 2 when query complexity and data volume justify it.

### 3.6 Data Flow Summary

```
                        WRITE PATH                                  READ PATH
                    ┌──────────────┐                         ┌──────────────┐
                    │ Python        │                         │ Python        │
                    │ Service       │                         │ Service       │
                    └──────┬───────┘                         └──────┬───────┘
                           │                                         │
                           ▼                                         ▼
                    ┌──────────────┐                         ┌──────────────┐
                    │ L1: Cache    │                         │ L1: Cache    │
                    │ (invalidate) │                         │ (lookup)     │
                    └──────┬───────┘                         └──┬───┬───────┘
                           │ hit? → no action                   │   │
                           ▼                                    │   ▼ miss
                    ┌──────────────┐                            │ ┌──────────────┐
                    │ L2: RocksDB  │◄──── WAL ──── fsync        │ │ L2: RocksDB  │
                    │ (write SST)  │                            │ │ (read)       │
                    └──┬───┬───┬───┘                            │ └──────┬───────┘
                       │   │   │                                │        │
                       ▼   ▼   ▼                                │        ▼
                    ┌───┐ ┌──┐ ┌──┐                             │  ┌──────────────┐
                    │L3 │ │L4 │ │L5│ (async)                    │  │ Return       │
                    │HNS│ │Tan│ │Dk│                             │  │ result       │
                    │W  │ │ti │ │DB│                             │  └──────────────┘
                    └───┘ └──┘ └──┘                             │
                                                                 │  Hybrid Search:
                                                                 │  ┌──────────────┐
                                                                 │  │ L3 HNSW  ×  │
                                                                 │  │ L4 Tantivy  │
                                                                 │  │ merge+rerank │
                                                                 │  └──────────────┘
```

---

## 4. Module Architecture

### 4.1 Rust Core Modules

```
contexter-core/
├── lib.rs                        # PyO3 module entry point
├── bridge.rs                     # #[pyclass]/#[pyfunction] declarations
│
├── cache/
│   ├── mod.rs                    # Cache trait + LRU entry
│   ├── dashmap_lru.rs            # DashMap + LRU eviction implementation
│   └── metrics.rs                # Cache hit/miss counters
│
├── storage/
│   ├── mod.rs                    # StorageBackend trait
│   ├── rocksdb.rs                # RocksDB implementation
│   ├── column_families.rs        # CF definitions, key encoding, per-CF config
│   ├── migrations.rs             # Schema version tracking + CF creation
│   └── types.rs                  # Row → Rust struct deserialization
│
├── vector/
│   ├── mod.rs                    # VectorIndex trait
│   ├── hnsw.rs                   # HNSW graph implementation
│   ├── distance.rs               # Cosine, Euclidean, Dot product kernels
│   └── snapshot.rs               # Persist/load binary snapshot
│
├── fts/
│   ├── mod.rs                    # FullTextSearch trait
│   ├── tantivy.rs                # Tantivy index wrapper
│   ├── schema.rs                 # Index schema definitions per entity type
│   └── query.rs                  # Query parsing + boosting
│
├── analytics/
│   ├── mod.rs                    # AnalyticsEngine trait
│   ├── duckdb.rs                 # DuckDB wrapper
│   ├── queries.rs                # Predefined SQL queries
│   └── sync.rs                   # RocksDB → DuckDB data transfer
│
├── compression/
│   ├── mod.rs                    # Compression trait
│   └── codecs.rs                 # Zstd / LZ4 wrappers
│
├── crdt/
│   ├── mod.rs                    # CRDT trait + LWW-Register
│   └── merge.rs                  # Conflict resolution + merge logic
│
├── versioning/
│   ├── mod.rs                    # ContentAddressedStore
│   ├── store.rs                  # Hash → blob storage
│   ├── gc.rs                     # Reference counting + sweep
│   └── diff.rs                   # Line-level diff (via `similar` crate)
│
├── models/
│   ├── mod.rs                    # Shared data model types
│   ├── memory.rs                 # Memory entity
│   ├── session.rs                # Session entity
│   ├── agent.rs                  # Agent entity
│   ├── skill.rs                  # Skill entity
│   ├── settings.rs               # Settings
│   ├── audit.rs                  # Audit log entry
│   ├── telemetry.rs              # Observability event
│   ├── notification.rs           # Notification record
│   ├── feedback.rs               # Bug report / suggestion
│   ├── correlation.rs            # Cross-session correlation types
│   └── analytics.rs              # Aggregated analytics types
│
├── engine/
│   ├── mod.rs                    # Top-level Engine struct (composes all tiers)
│   ├── session.rs                # Session lifecycle operations
│   ├── memory.rs                 # Memory CRUD + hybrid search
│   ├── agent.rs                  # Agent registry operations
│   ├── skill.rs                  # Skill registry operations
│   ├── search.rs                 # Unified search (L3+L4 hybrid)
│   ├── export.rs                 # Data export logic
│   └── analytics.rs              # Analytics computation
│
├── wal/
│   └── mod.rs                    # Thin wrapper over RocksDB built-in WAL
│                                (recovery, checkpoint, replay utilities)
│
├── telemetry/
│   ├── mod.rs                    # Self-observability instrumentation
│   ├── metrics.rs                # Counter, gauge, histogram types
│   ├── reporter.rs               # Periodic metric snapshot to L2 telemetry CF
│   └── tracing.rs                # tracing-subscriber integration
│
└── util/
    ├── mod.rs
    ├── id.rs                     # UUID v7 generation
    └── time.rs                   # Timestamp utilities
```

### 4.2 Python Layer Modules

```
contexter-server/
├── main.py                       # FastAPI app + startup
├── mcp_server.py                 # FastMCP server entry point
│
├── api/                          # REST endpoints
│   ├── __init__.py
│   ├── sessions.py               # /api/sessions endpoints
│   ├── memories.py               # /api/memories endpoints
│   ├── agents.py                 # /api/agents endpoints
│   ├── skills.py                 # /api/skills endpoints
│   ├── analytics.py              # /api/analytics endpoints
│   ├── search.py                 # /api/search endpoints
│   ├── export.py                 # /api/export endpoints
│   ├── settings.py               # /api/settings endpoints
│   ├── notifications.py          # /api/notifications endpoints
│   ├── feedback.py               # /api/feedback endpoints
│   ├── files.py                  # /api/files (versioning) endpoints
│   ├── audit.py                  # /api/audit endpoints
│   ├── correlation.py            # /api/correlation endpoints
│   └── onboarding.py             # /api/onboarding endpoints
│
├── mcp_tools/                    # MCP tool definitions
│   ├── __init__.py
│   ├── sessions.py               # MCP tool: session CRUD
│   ├── memories.py               # MCP tool: memory CRUD + search
│   ├── agents.py                 # MCP tool: agent CRUD
│   ├── skills.py                 # MCP tool: skill CRUD
│   └── system.py                 # MCP tool: system info, settings
│
├── services/                     # Business logic (orchestration)
│   ├── __init__.py
│   ├── session_service.py        # Session lifecycle
│   ├── memory_service.py         # Memory CRUD + vector search
│   ├── agent_service.py          # Agent registry logic
│   ├── skill_service.py          # Skill registry logic
│   ├── analytics_service.py      # Efficiency, correlation, cost calc
│   ├── search_service.py         # Unified search orchestration
│   ├── export_service.py         # Data export/report generation
│   ├── notification_service.py   # In-app + webhook dispatch
│   ├── audit_service.py          # Audit logging
│   ├── file_watcher.py           # Auto-versioning watcher (delegates to Rust notify)
│   ├── onboarding_service.py     # First-run wizard state
│   ├── correlation_service.py    # Cross-session correlation logic
│   └── settings_service.py       # Configuration management
│
├── models/                       # Pydantic models (mirror Rust models)
│   ├── __init__.py
│   ├── session.py
│   ├── memory.py
│   ├── agent.py
│   ├── skill.py
│   ├── analytics.py
│   ├── settings.py
│   ├── audit.py
│   ├── search.py
│   ├── export.py
│   ├── correlation.py
│   └── notifications.py
│
├── core/                         # Core bridge + utilities
│   ├── __init__.py
│   └── bridge.py                 # Wraps Rust `contexter_core` module
│                                Provides sync + async access patterns.
│                                Sync for CLI/scripts, async via
│                                asyncio.to_thread() for API layer.
│
└── cli/                          # CLI interface (for admin/maintenance)
    ├── __init__.py
    └── main.py                   # Click/typer CLI for diagnostics, export, GC
```

### 4.3 React UI Modules

```
contexter-web/
├── src/
│   ├── main.tsx
│   ├── App.tsx                    # Router + AppShell
│   │
│   ├── api/
│   │   ├── client.ts              # Auto-generated API client (openapi-generator or orval)
│   │   └── hooks/                 # React Query hooks per domain
│   │       ├── useSessions.ts
│   │       ├── useMemories.ts
│   │       ├── useAgents.ts
│   │       ├── useSkills.ts
│   │       ├── useAnalytics.ts
│   │       └── ...
│   │
│   ├── components/
│   │   ├── layout/                # AppShell, SidebarNav, TopBar, Breadcrumb
│   │   ├── shared/                # StatCard, DataTable, Tag, FilterBar, TimeframeSelector
│   │   ├── charts/                # Reusable chart wrappers (TimeSeriesChart, PieChart, BarChart)
│   │   ├── common/                # Modal, Toast, LoadingSkeleton, EmptyState, ErrorBoundary
│   │   └── search/                # GlobalSearch palette, SearchResultCard
│   │
│   ├── pages/
│   │   ├── Dashboard/
│   │   ├── Sessions/              # Session Manager + Session Detail
│   │   ├── Memories/              # Memory Explorer + Memory Detail
│   │   ├── Agents/                # Agent Registry + Agent Detail
│   │   ├── Skills/                # Skill Registry + Skill Detail
│   │   ├── Efficiency/            # Efficiency Mapper + 6 detail pages
│   │   ├── Analytics/             # Overview + 6 detail pages (incl. Model Detail)
│   │   ├── Settings/              # 8 settings sections
│   │   ├── Notifications/         # Notification Center
│   │   ├── Feedback/              # Feedback & Bug Reporter (3 tabs)
│   │   ├── Onboarding/            # First-run wizard
│   │   ├── Playground/            # API Playground (REST + MCP tabs)
│   │   ├── Search/                # Dedicated search page
│   │   ├── Exports/               # Data Export & Reports (3 tabs)
│   │   ├── Correlation/           # Cross-Session Correlation (3 tabs)
│   │   └── Audit/                 # Versioning & Audit Trail (3 tabs)
│   │
│   ├── hooks/                     # Shared custom hooks
│   ├── types/                     # TypeScript types (mirrors Pydantic)
│   └── utils/                     # Formatters, date helpers, constants
```

---

## 5. Data Model

### 5.1 Entity Relationship Diagram

```
┌──────────┐       ┌──────────────┐       ┌───────────┐
│  Session  │──────▶│   Memory     │──────▶│  File     │
│           │       │              │       │ (Version) │
└──────────┘       └──────────────┘       └───────────┘
     │                                         │
     │                                         │
     ▼                                         ▼
┌──────────┐       ┌──────────────┐       ┌────────────┐
│  Agent   │       │    Skill     │       │  Settings  │
└──────────┘       └──────────────┘       └────────────┘

┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│  Audit Log   │  │ Notification │  │  Feedback    │
└──────────────┘  └──────────────┘  └──────────────┘

┌──────────────────┐  ┌──────────────────┐
│  Analytics/Aggr  │  │  Correlation     │
└──────────────────┘  └──────────────────┘
```

### 5.2 Core Entities

All entities use **UUID v7** as primary keys (time-ordered, sortable). All timestamps are **UTC** (`chrono::Utc` in Rust, `datetime.utcnow()` in Python).

#### Session
| Field | Type | Description |
|---|---|---|
| `id` | UUID v7 | Primary key |
| `project` | String | Project namespace |
| `agent_id` | UUID v7 | Agent that owns this session |
| `status` | Enum(active, completed, error) | Current state |
| `turn_count` | u32 | Number of message turns |
| `duration_ms` | u64 | Total active duration |
| `efficiency_score` | f32 | Computed on session close (0.0–1.0) |
| `metadata` | JSON | Arbitrary key-value data |
| `created_at` | DateTime | |
| `last_active` | DateTime | |

#### Memory
| Field | Type | Description |
|---|---|---|
| `id` | UUID v7 | Primary key |
| `session_id` | UUID v7 | Parent session |
| `agent_id` | UUID v7 | Creating agent |
| `type` | Enum(fact, preference, procedure, context, episode) | Memory category |
| `content` | Text | Memory text |
| `embedding` | f32[] | Vector embedding (optional, computed async) |
| `tags` | String[] | Categorization tags |
| `version` | u32 | Current version number |
| `created_at` | DateTime | |
| `updated_at` | DateTime | |

#### Agent
| Field | Type | Description |
|---|---|---|
| `id` | UUID v7 | Primary key |
| `name` | String | Unique name |
| `type` | String | Agent class/category |
| `description` | String | Purpose and capabilities |
| `capabilities` | String[] | Feature tags |
| `status` | Enum(active, inactive) | |
| `config` | JSON | Agent-specific configuration |
| `version` | u32 | Definition version |
| `created_at` | DateTime | |
| `updated_at` | DateTime | |

#### Skill
| Field | Type | Description |
|---|---|---|
| `id` | UUID v7 | Primary key |
| `name` | String | Unique name |
| `description` | String | Purpose |
| `category` | String | Functional category |
| `version` | u32 | Definition version |
| `file_path` | String | Path to skill definition file |
| `created_at` | DateTime | |
| `updated_at` | DateTime | |

#### File Version
| Field | Type | Description |
|---|---|---|
| `id` | UUID v7 | Primary key |
| `file_path` | String | Absolute path to tracked file |
| `version` | u32 | Monotonic version number |
| `content_hash` | SHA-256 hex | Content-addressed identifier |
| `size_bytes` | u64 | |
| `actor` | String | "auto-save" or human/agent identifier |
| `created_at` | DateTime | |

#### Audit Log Entry
| Field | Type | Description |
|---|---|---|
| `id` | UUID v7 | Primary key |
| `entity_type` | Enum(session, memory, agent, skill, settings, file) | |
| `entity_id` | UUID v7 | |
| `action` | Enum(created, updated, deleted, versioned, exported) | |
| `actor` | String | Agent name or "human" |
| `summary` | String | Human-readable description |
| `metadata` | JSON | Before/after state diff (optional) |
| `created_at` | DateTime | |

#### Telemetry Event
| Field | Type | Description |
|---|---|---|
| `id` | UUID v7 | Primary key |
| `event_type` | String | e.g., "cache_hit", "api_latency", "search_query" |
| `scope` | String | e.g., "engine.memory.read", "api.sessions.list" |
| `value` | f64 | Numeric measurement |
| `labels` | JSON | Key-value dimensions (entity_type, status, etc.) |
| `timestamp` | DateTime | |

*Note: Full Pydantic/serde models for all entities and their request/response schemas are in the respective module files.*

---

## 6. Storage Engine

### 6.1 StorageBackend Trait (Rust)

The `StorageBackend` trait provides a unified interface for RocksDB (default) and future backends (PostgreSQL/pgvector, SQLite for embedded-light). Note that the trait is **synchronous** — RocksDB operations are inherently synchronous, and async wrapping is handled at the PyO3 bridge layer.

```rust
/// Unified storage backend trait. Default implementation: RocksDB.
/// All methods are synchronous. Python layer wraps via asyncio.to_thread().
pub trait StorageBackend: Send + Sync {
    // --- Session ---
    fn create_session(&self, session: NewSession) -> Result<Session>;
    fn get_session(&self, id: Uuid) -> Result<Option<Session>>;
    fn list_sessions(&self, filter: &SessionFilter) -> Result<Vec<Session>>;
    fn update_session(&self, id: Uuid, patch: &SessionPatch) -> Result<Session>;
    fn delete_session(&self, id: Uuid) -> Result<()>;
    fn count_sessions(&self, filter: &SessionFilter) -> Result<u64>;

    // --- Memory ---
    fn create_memory(&self, memory: NewMemory) -> Result<Memory>;
    fn get_memory(&self, id: Uuid) -> Result<Option<Memory>>;
    fn search_memories(&self, query: &MemorySearchQuery) -> Result<Vec<Memory>>;
    fn update_memory(&self, id: Uuid, patch: &MemoryPatch) -> Result<Memory>;
    fn delete_memory(&self, id: Uuid) -> Result<()>;
    fn count_memories(&self, filter: &MemoryFilter) -> Result<u64>;

    // --- Agent ---
    fn create_agent(&self, agent: NewAgent) -> Result<Agent>;
    fn get_agent(&self, id: Uuid) -> Result<Option<Agent>>;
    fn list_agents(&self, filter: &AgentFilter) -> Result<Vec<Agent>>;
    fn update_agent(&self, id: Uuid, patch: &AgentPatch) -> Result<Agent>;
    fn delete_agent(&self, id: Uuid) -> Result<()>;

    // --- Skill ---
    fn create_skill(&self, skill: NewSkill) -> Result<Skill>;
    fn get_skill(&self, id: Uuid) -> Result<Option<Skill>>;
    fn list_skills(&self, filter: &SkillFilter) -> Result<Vec<Skill>>;
    fn update_skill(&self, id: Uuid, patch: &SkillPatch) -> Result<Skill>;
    fn delete_skill(&self, id: Uuid) -> Result<()>;

    // --- Vector ---
    fn index_embedding(&self, memory_id: Uuid, embedding: &[f32]) -> Result<()>;
    fn knn_search(&self, query: &[f32], k: usize, filter: &VectorFilter) -> Result<Vec<ScoredMemoryId>>;

    // --- Full-Text Search ---
    fn fts_index(&self, memory_id: Uuid, content: &str, tags: &[String]) -> Result<()>;
    fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<ScoredMemoryId>>;

    // --- Settings ---
    fn get_setting(&self, key: &str) -> Result<Option<String>>;
    fn set_setting(&self, key: &str, value: &str) -> Result<()>;

    // --- Audit ---
    fn append_audit_entry(&self, entry: &NewAuditEntry) -> Result<()>;
    fn query_audit_log(&self, filter: &AuditFilter) -> Result<Vec<AuditEntry>>;

    // --- Maintenance ---
    fn flush(&self) -> Result<()>;
    fn checkpoint(&self) -> Result<u64>;
    fn replay_wal_since(&self, lsn: u64) -> Result<Vec<WalRecord>>;
    fn storage_size(&self) -> Result<StorageSize>;
}
```

### 6.2 RocksDB Implementation

The RocksDB implementation is the default and primary backend. Key configuration:

```rust
use rocksdb::{DB, Options, ColumnFamilyDescriptor, SliceTransform, BlockBasedOptions, Cache};

pub fn open_rocksdb(path: &Path) -> Result<DB> {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    opts.set_max_background_jobs(4);
    opts.set_bytes_per_sync(1_048_576);  // 1MB

    let block_cache = Cache::new_lru_cache(256 * 1024 * 1024);  // 256MB

    let cfs: Vec<ColumnFamilyDescriptor> = vec![
        ColumnFamilyDescriptor::new("memory_items", {
            let mut cf_opts = Options::default();
            cf_opts.set_compression_type(rocksdb::DBCompressionType::Zstd);
            cf_opts.compression_opts(3);   // level 3
            cf_opts.set_write_buffer_size(64 * 1024 * 1024);
            let mut bbt_opts = BlockBasedOptions::default();
            bbt_opts.set_block_cache(&block_cache);
            cf_opts.set_block_based_table_factory(&bbt_opts);
            cf_opts
        }),
        ColumnFamilyDescriptor::new("sessions", {
            let mut cf_opts = Options::default();
            cf_opts.set_compression_type(rocksdb::DBCompressionType::Zstd);
            cf_opts.compression_opts(3);
            cf_opts.set_write_buffer_size(32 * 1024 * 1024);
            cf_opts
        }),
        ColumnFamilyDescriptor::new("agents", {
            let mut cf_opts = Options::default();
            cf_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
            cf_opts.set_write_buffer_size(16 * 1024 * 1024);
            cf_opts
        }),
        ColumnFamilyDescriptor::new("skills", /* LZ4, 16MB */ ...),
        ColumnFamilyDescriptor::new("efficiency_map", /* LZ4, 8MB */ ...),
        ColumnFamilyDescriptor::new("telemetry", /* LZ4, 4MB */ ...),
        ColumnFamilyDescriptor::new("conflicts", /* Zstd, 8MB */ ...),
        ColumnFamilyDescriptor::new("index_state", /* LZ4, 4MB */ ...),
    ];

    DB::open_cf_descriptors(&opts, path, cfs)
}
```

### 6.3 PostgreSQL Implementation (Future)

A future `StorageBackend` implementation for PostgreSQL will:
- Use `sqlx` with connection pooling
- Support `pgvector` extension for vector search (falling back to Rust HNSW otherwise)
- Use PostgreSQL's native WAL for replication (future multi-node support)
- Share the same `StorageBackend` trait, so no orchestration code changes

---

## 7. PyO3 Bridge

### 7.1 Bridge Architecture

The Python layer never calls Rust struct methods directly. Instead, a single `Engine` struct composes all tiers (cache, storage, vector, fts, analytics) and is exposed via PyO3 as a `#[pyclass]`:

```rust
// lib.rs — the sole PyO3 entry point
#[pyclass]
pub struct Engine {
    cache: Arc<DashMapCache>,
    storage: Arc<RwLock<Box<dyn StorageBackend>>>,
    vector_index: Arc<RwLock<VectorIndex>>,
    fts_index: Arc<RwLock<FullTextIndex>>,
    analytics: Arc<RwLock<AnalyticsEngine>>,
    telemetry: Arc<TelemetryCollector>,
}

#[pymethods]
impl Engine {
    #[new]
    pub fn new(config: PyStorageConfig) -> PyResult<Self> { ... }

    // Session
    pub fn create_session(&self, data: PyNewSession) -> PyResult<PySession> { ... }
    pub fn get_session(&self, id: &str) -> PyResult<Option<PySession>> { ... }
    pub fn list_sessions(&self, filter: PySessionFilter) -> PyResult<Vec<PySession>> { ... }

    // Memory
    pub fn create_memory(&self, data: PyNewMemory) -> PyResult<PyMemory> { ... }
    pub fn search_memories(&self, query: PySearchQuery) -> PyResult<PySearchResults> { ... }

    // Generic key-value store (for settings, audit, etc.)
    pub fn store(&self, cf: &str, key: &str, value: &str) -> PyResult<()> { ... }
    pub fn get(&self, cf: &str, key: &str) -> PyResult<Option<String>> { ... }

    // Maintenance
    pub fn checkpoint(&self) -> PyResult<u64> { ... }
    pub fn storage_size(&self) -> PyResult<PyStorageSize> { ... }
    pub fn telemetry_snapshot(&self) -> PyResult<Vec<PyTelemetryEvent>> { ... }
}
```

Python-side wrapper:

```python
# contexter/core/bridge.py
from contexter_core import Engine as RustEngine
from typing import Any
import asyncio
from concurrent.futures import ThreadPoolExecutor

_executor = ThreadPoolExecutor(max_workers=4)

class StorageEngine:
    """Async-safe wrapper around the Rust Engine."""

    def __init__(self, config: dict):
        self._rust = RustEngine(_to_py_config(config))

    async def create_memory(self, data: dict) -> dict:
        return await asyncio.to_thread(self._rust.create_memory, data)

    async def search_memories(self, query: dict) -> dict:
        return await asyncio.to_thread(self._rust.search_memories, query)

    async def store(self, cf: str, key: str, value: str) -> None:
        return await asyncio.to_thread(self._rust.store, cf, key, value)

    async def get(self, cf: str, key: str) -> str | None:
        return await asyncio.to_thread(self._rust.get, cf, key)
    # ...
```

### 7.2 Concurrency Model

- **Rust core** uses `Arc<RwLock<...>>` for shared mutable state (storage, vector index, FTS index). Reads take read locks, writes take write locks — contention is low because RocksDB operations are fast (microseconds).
- **Python async code** calls bridge functions via `asyncio.to_thread()` since PyO3 calls release the GIL but are synchronous.
- **Thread pool** (`ThreadPoolExecutor(max_workers=4)`) prevents sequential bottleneck. Four workers are sufficient for I/O-bound RocksDB operations.
- **Cache** (`DashMap`) is lock-free for reads — no `RwLock` needed for the hot path.
- **Telemetry** uses a dedicated `AtomicU64` + periodic flush pattern — no locks on the hot increment path.

### 7.3 Serde Across the Bridge

Python ↔ Rust data flows through JSON-serialized dicts:

```
Python dict → json.dumps → str → PyO3 → serde_json::from_str → Rust struct
Python dict ← json.loads ← str ← PyO3 ← serde_json::to_string ← Rust struct
```

This avoids complex PyO3 type mapping while keeping the boundary explicit. For high-throughput paths (batch memory writes, telemetry ingestion), the bridge can be optimized with direct PyO3 `PyAny` conversions in Phase 2.

---

## 8. CRDT & Versioning

### 8.1 Conflict Resolution (LWW-Register)

Concurrent edits to the same entity (e.g., two agents updating the same memory) are resolved via Last-Writer-Wins:

- Each write carries a **logical timestamp** (monotonic counter per entity type) + **wall clock timestamp**.
- On conflict detection (same key, different writers, same wall clock tick):
  1. The conflict is recorded in the `conflicts` column family.
  2. The write with the higher logical timestamp wins.
  3. The losing write's content is preserved as a conflict record for manual review.
- Conflict records are surfaced in the UI under Settings > Data Management > Conflict Resolution.

### 8.2 Content-Addressed Versioning

File versioning (for auto-tracked files like AGENTS.md, SPEC.md) uses a content-addressed store:

- Content → `SHA-256(content)` → blob stored in `~/.contexter/content_store/{prefix}/{hash}`
- Reference counting: each `FileVersion` record increments ref count for its hash
- On version deletion: decrement ref count. GC sweep removes blobs with `ref_count == 0`
- Deduplication: identical content produces the same hash → no duplicate storage

### 8.3 Diff Computation

Version comparisons use the `similar` crate for line-level diffs:

```rust
pub fn compute_diff(old: &str, new: &str) -> DiffResult {
    let changeset = similar::TextDiff::from_lines(old, new);
    DiffResult {
        additions: changeset.inserted_count(),
        deletions: changeset.deleted_count(),
        hunks: changeset
            .iter_all_changes()
            .map(|c| DiffHunk {
                tag: match c.tag() {
                    similar::ChangeTag::Equal => "equal",
                    similar::ChangeTag::Insert => "insert",
                    similar::ChangeTag::Delete => "delete",
                },
                value: c.value().to_string(),
                old_line: c.old_index().map(|i| i as u32 + 1),
                new_line: c.new_index().map(|i| i as u32 + 1),
            })
            .collect(),
    }
}
```

---

## 9. Compression Strategy

Compression is configured **per column family** to balance space savings vs. read/write speed:

| CF | Algorithm | Level | Rationale |
|---|---|---|---|
| `memory_items` | Zstd | 3 | Content-heavy. Zstd-3 gives ~3-5x compression on text with minimal CPU overhead. |
| `sessions` | Zstd | 3 | Session JSON with turn history can be large. |
| `agents` | LZ4 | default | Small documents, read-frequent. LZ4 is fastest for read. |
| `skills` | LZ4 | default | Same as agents. |
| `efficiency_map` | LZ4 | default | Float arrays, already compact. Read-frequent. |
| `telemetry` | LZ4 | default | Very high write volume. Speed > ratio. |
| `conflicts` | Zstd | 1 | Low volume, wants durability. Zstd-1 is faster than Zstd-3. |
| `index_state` | LZ4 | default | Tiny records, point lookups only. |

Additionally:
- **Vector index snapshot** (`vector_index.bin`): Zstd level 6 (compressed once on write, decompressed once on load)
- **Tantivy index** (`tantivy_index/`): Tantivy's own compression (no additional layer)
- **Export archives**: Zstd level 10 (cold, one-time compression/decompression)

---

## 10. Analytics Engine

### 10.1 DuckDB Integration

The analytics engine uses DuckDB in-process for columnar analytical queries. Data flows:

1. **On analytics request**: Python service calls `analytics_engine.query(sql, params)`
2. **Data source**: Queries against inline data aggregated from RocksDB iterators
3. **Caching**: Results are cached in L2 `efficiency_map` CF for configurable TTL (default: 5 minutes)
4. **Refresh**: Cache invalidated when relevant data changes (memory write, session close)

### 10.2 Efficiency Calculation

```
session_efficiency = completed_tasks / total_tasks × avg_spec_adherence
  where:
    completed_tasks = count of ACs marked passed by Validators
    total_tasks = count of all ACs in the session
    avg_spec_adherence = % of SPEC requirements with corresponding implementation
```

### 10.3 Correlation Engine

- Computes Pearson/Spearman correlation between metric pairs
  (e.g., `turn_count` vs `efficiency`, `duration` vs `memories_created`)
- Uses session-level aggregates from RocksDB
- Results cached in `efficiency_map` CF, recomputed on data change or explicit refresh

### 10.4 Cost Tracking

- Token counts per session per model (input + output)
- Cost per model configurable in Settings > LLM Providers ($/1K tokens)
- Daily/weekly/monthly aggregation for Cost & Token Analytics (L5 DuckDB queries or L2 iterators)

---

## 11. API Surface

### 11.1 REST API (FastAPI — port 8000)

All endpoints under `/api/v1/`:

| Group | Endpoints |
|---|---|
| Sessions | `GET/POST /sessions`, `GET/PUT/DELETE /sessions/:id`, `POST /sessions/:id/resume` |
| Memories | `GET/POST /memories`, `GET/PUT/DELETE /memories/:id`, `GET /memories/search`, `POST /memories/:id/versions` |
| Agents | `GET/POST /agents`, `GET/PUT/DELETE /agents/:id` |
| Skills | `GET/POST /skills`, `GET/PUT/DELETE /skills/:id` |
| Analytics | `GET /analytics/overview`, `GET /analytics/health`, `GET /analytics/performance`, `GET /analytics/resources`, `GET /analytics/costs`, `GET /analytics/costs/models/:id`, `GET /analytics/services` |
| Efficiency | `GET /efficiency/overview`, `GET /efficiency/memory`, `GET /efficiency/sessions`, `GET /efficiency/agents`, `GET /efficiency/skills`, `GET /efficiency/tokens`, `GET /efficiency/correlation` |
| Search | `GET /search?q=&type=&project=&page=&limit=` |
| Settings | `GET/PUT /settings/:section` |
| Notifications | `GET /notifications`, `PUT /notifications/:id/read`, `POST /notifications/read-all` |
| Audit | `GET /audit?entity_type=&action=&actor=&q=&limit=&offset=` |
| Files | `GET /files?path=`, `GET /files/:hash/diff?base=&compare=`, `POST /files/watch` |
| Correlation | `GET /correlation/overview?timeframe=`, `GET /correlation/timeline?project=&agent=`, `GET /correlation/compare?a=&b=` |
| Export | `POST /export/submit`, `GET /export/status/:id`, `GET /export/download/:id`, `GET /export/history` |
| Feedback | `POST /feedback/bug`, `POST /feedback/suggest`, `GET /changelog` |
| Onboarding | `GET /onboarding/status`, `POST /onboarding/wizard`, `GET /onboarding/progress` |

### 11.2 MCP Server (FastMCP — port 8001)

**Tools:**

| Tool | Description |
|---|---|
| `store_memory` | Store a memory with optional type, tags, embedding |
| `search_memories` | Semantic + keyword search across memories |
| `get_session` | Retrieve session with full context |
| `list_recent_sessions` | List recent sessions with status, agent, duration |
| `get_agent_info` | Get agent details and capabilities |
| `list_skills` | List available skills with effectiveness |
| `get_system_health` | Get system health metrics |
| `export_data` | Trigger data export |

**Resources:**

| Resource URI | Description |
|---|---|
| `contexter://session/{id}` | Full session context |
| `contexter://memory/{id}` | Memory content with metadata |
| `contexter://agent/{id}` | Agent definition |
| `contexter://analytics/overview` | Analytics summary |

---

## 12. Configuration

### 12.1 Data Directory

```
~/.contexter/
├── data/                          # RocksDB database directory
│   ├── memory_items/              # CF data (actually all CFs share the rocksdb dir)
│   ├── ...                        # RocksDB manages CF files internally
│   ├── CURRENT
│   ├── MANIFEST-*
│   ├── OPTIONS-*
│   └── WAL/                       # RocksDB WAL files
│
├── vector_index.bin               # HNSW vector index snapshot
├── tantivy_index/                 # Tantivy full-text index directory
├── content_store/                 # Content-addressed versioned blobs
│   └── ab/
│       └── cdef...                # SHA-256 prefixed blob files
│
├── exports/                       # Generated export archives (.json.zst)
└── config.yaml                    # User settings
```

### 12.2 config.yaml

```yaml
project:
  name: "Contexter"
  default_project: "default"

storage:
  path: "~/.contexter/"
  engine: "rocksdb"         # "rocksdb" only in v1; "postgres" future
  postgres_url: ""          # optional, for future PostgreSQL backend

cache:
  max_entries_per_type: 10000
  lru_eviction_batch: 100

mcp_server:
  host: "127.0.0.1"
  port: 8001
  auth_token: ""

llm_providers:
  - name: "openai"
    api_key: ""
    models: ["gpt-4", "gpt-3.5-turbo"]

notifications:
  in_app: true
  email: ""
  webhook_url: ""

versioning:
  tracked_files:
    - "/path/to/AGENTS.md"
    - "/path/to/SPEC.md"

analytics:
  enabled: true
  retention_days: 90

telemetry:
  enabled: true
  snapshot_interval_secs: 60
```

---

## 13. Test Architecture

Tests **mirror source structure** across all three codebases. This is a binding design decision — every source module has a corresponding test module at the same path.

### 13.1 Rust Core Tests (`contexter-core`)

| Test Level | Location | Pattern | What It Tests |
|---|---|---|---|
| **Unit tests** | Inline `#[cfg(test)] mod tests` per `.rs` file | One `mod tests` per source file | Individual functions, edge cases, error paths |
| **Integration tests** | `tests/` directory mirrors `src/` | `tests/storage/`, `tests/vector/`, `tests/engine/`, `tests/cache/`, `tests/fts/`, `tests/analytics/`, `tests/bridges/pyo3/` | Module-level behavior, cross-module interaction, serialization round-trips |

```
contexter-core/
├── src/
│   ├── storage/
│   │   ├── mod.rs          ← #[cfg(test)] mod tests { ... }
│   │   ├── rocksdb.rs      ← #[cfg(test)] mod tests { ... }
│   │   ├── column_families.rs ← #[cfg(test)] mod tests { ... }
│   │   └── migrations.rs   ← #[cfg(test)] mod tests { ... }
│   ├── engine/
│   │   ├── memory.rs       ← #[cfg(test)] mod tests { ... }
│   │   ├── session.rs      ← #[cfg(test)] mod tests { ... }
│   │   └── search.rs       ← #[cfg(test)] mod tests { ... }
│   ├── cache/
│   │   └── dashmap_lru.rs  ← #[cfg(test)] mod tests { ... }
│   ├── ... (every .rs file has inline tests)
│
├── tests/
│   ├── storage/
│   │   ├── mod.rs          # Test helpers: temp dirs, sample data
│   │   ├── rocksdb_test.rs # Full CRUD, WAL replay, compaction
│   │   └── column_families_test.rs
│   ├── vector/
│   │   ├── hnsw_test.rs    # Accuracy, persistence, recovery
│   │   └── distance_test.rs
│   ├── cache/
│   │   └── lru_test.rs     # Eviction, concurrency, write-through
│   ├── fts/
│   │   └── tantivy_test.rs # Indexing, search, field boosting
│   ├── analytics/
│   │   └── duckdb_test.rs  # Query correctness, sync from RocksDB
│   ├── compression/
│   │   └── codecs_test.rs  # Round-trip Zstd/LZ4, decompress error
│   ├── engine/
│   │   ├── memory_test.rs  # Full memory lifecycle via Engine
│   │   ├── session_test.rs # Session lifecycle + checkpoint
│   │   └── search_test.rs  # Hybrid search (L3+L4 merge+rerank)
│   ├── bridges/
│   │   └── pyo3_test.rs    # PyO3 type mapping, JSON round-trip
│   └── common/
│       ├── mod.rs          # Shared test helpers (tempdir, sample data generators)
│       └── fixtures.rs     # Reusable test data constants
```

**Test utilities:**
- `tests/common/mod.rs` provides:
  - `TempRocksDb::new()` — creates a temporary RocksDB instance for testing
  - `sample_memory()`, `sample_session()`, etc. — reusable test data generators
  - `assert_storage_size()` — verifies storage metrics
- Integration tests run with `#[serial]` when using shared resources (rare — temp dirs are unique)

### 13.2 Python Layer Tests (`contexter-server`)

| Test Level | Location | Pattern | What It Tests |
|---|---|---|---|
| **Service tests** | `tests/services/` | Mocked `StorageEngine` (Python mock of Rust bridge) | Business logic, validation, error handling |
| **API tests** | `tests/api/` | FastAPI TestClient | HTTP endpoints, auth, serialization, error codes |
| **MCP tests** | `tests/mcp/` | MCP client test harness | Tool invocation, resource resolution |
| **Model tests** | `tests/models/` | Pydantic validation | Schema constraints, serialization, type coercion |
| **CLI tests** | `tests/cli/` | Click/typer test runner | Command execution, output format |

```
contexter-server/
├── src/
│   ├── services/
│   │   ├── session_service.py
│   │   └── memory_service.py
│   ├── api/
│   │   ├── sessions.py
│   │   └── memories.py
│   └── ... (source code)
│
├── tests/
│   ├── conftest.py              # Shared fixtures: mock_storage_engine, test_client, sample_data
│   │
│   ├── services/
│   │   ├── test_session_service.py
│   │   ├── test_memory_service.py
│   │   ├── test_agent_service.py
│   │   ├── test_skill_service.py
│   │   ├── test_search_service.py
│   │   ├── test_analytics_service.py
│   │   ├── test_export_service.py
│   │   ├── test_notification_service.py
│   │   ├── test_audit_service.py
│   │   ├── test_file_watcher.py
│   │   ├── test_correlation_service.py
│   │   └── test_onboarding_service.py
│   │
│   ├── api/
│   │   ├── test_sessions.py     # TestClient + mock bridge
│   │   ├── test_memories.py
│   │   ├── test_agents.py
│   │   ├── test_skills.py
│   │   ├── test_analytics.py
│   │   ├── test_search.py
│   │   ├── test_export.py
│   │   ├── test_settings.py
│   │   ├── test_notifications.py
│   │   ├── test_feedback.py
│   │   ├── test_files.py
│   │   ├── test_audit.py
│   │   ├── test_correlation.py
│   │   └── test_onboarding.py
│   │
│   ├── mcp/
│   │   ├── test_mcp_sessions.py
│   │   ├── test_mcp_memories.py
│   │   ├── test_mcp_agents.py
│   │   ├── test_mcp_skills.py
│   │   └── test_mcp_system.py
│   │
│   ├── models/
│   │   ├── test_session_model.py
│   │   ├── test_memory_model.py
│   │   ├── test_agent_model.py
│   │   ├── test_skill_model.py
│   │   ├── test_analytics_model.py
│   │   ├── test_settings_model.py
│   │   ├── test_audit_model.py
│   │   └── test_search_model.py
│   │
│   └── cli/
│       ├── test_export_cli.py
│       └── test_diagnostics_cli.py
```

**conftest.py fixtures:**

```python
@pytest.fixture
def mock_storage_engine():
    """Returns a MockStorageEngine that wraps a temporary Rust Engine if available,
    or a pure-Python mock if the Rust module isn't compiled."""
    ...

@pytest.fixture
def test_client(mock_storage_engine):
    """FastAPI TestClient with mocked storage."""
    app.dependency_overrides[get_storage_engine] = lambda: mock_storage_engine
    yield TestClient(app)
    app.dependency_overrides.clear()

@pytest.fixture
def sample_session():
    return {"id": str(uuid7()), "project": "test", "status": "active", ...}
```

### 13.3 React UI Tests (`contexter-web`)

| Test Level | Location | Pattern | What It Tests |
|---|---|---|---|
| **Component tests** | `tests/components/` | Vitest + Testing Library | Component rendering, state changes, user events |
| **Hook tests** | `tests/hooks/` | Vitest + renderHook | Custom hooks, React Query integration |
| **API client tests** | `tests/api/` | MSW (Mock Service Worker) | API client behavior, error handling, retry logic |
| **Store tests** | `tests/store/` | Vitest | State management (Zustand or context reducers) |
| **Page tests** | `tests/routes/` | Vitest + MSW | Full page rendering, navigation, data loading |

```
contexter-web/
├── src/
│   ├── components/
│   │   ├── shared/
│   │   │   ├── StatCard.tsx
│   │   │   └── DataTable.tsx
│   │   └── ...
│   └── pages/
│       ├── Dashboard/
│       ├── Sessions/
│       └── ... (source code)
│
├── tests/
│   ├── setup.ts                    # Vitest setup: MSW server, global mocks
│   │
│   ├── components/
│   │   ├── shared/
│   │   │   ├── StatCard.test.tsx
│   │   │   └── DataTable.test.tsx
│   │   └── common/
│   │       ├── Modal.test.tsx
│   │       ├── EmptyState.test.tsx
│   │       └── TimeframeSelector.test.tsx
│   │
│   ├── hooks/
│   │   ├── useSessions.test.ts
│   │   ├── useMemories.test.ts
│   │   └── useAnalytics.test.ts
│   │
│   ├── api/
│   │   ├── client.test.ts
│   │   └── error-handling.test.ts
│   │
│   ├── store/
│   │   ├── session-store.test.ts
│   │   └── ui-store.test.ts
│   │
│   └── routes/
│       ├── Dashboard.test.tsx
│       ├── SessionManager.test.tsx
│       ├── SessionDetail.test.tsx
│       ├── MemoryExplorer.test.tsx
│       ├── MemoryDetail.test.tsx
│       ├── AgentRegistry.test.tsx
│       ├── AgentDetail.test.tsx
│       ├── SkillRegistry.test.tsx
│       ├── SkillDetail.test.tsx
│       ├── EfficiencyMapper.test.tsx
│       ├── AnalyticsOverview.test.tsx
│       ├── Settings.test.tsx
│       └── ... (one per page)
```

**Test configuration:**
- `tests/setup.ts` sets up MSW handlers for all API endpoints
- All API calls are intercepted — no real backend needed for UI tests
- React Query cache is cleared between tests
- Framer Motion is disabled in test environment for deterministic rendering

---

## 14. Self-Observability

The system instruments itself with metrics that are visible in the UI's System Health and Performance Trends pages.

### 14.1 Collected Metrics

| Metric | Type | Labels | Source |
|---|---|---|---|
| `cache_hit_ratio` | Gauge (0.0–1.0) | entity_type | L1 Cache |
| `storage_read_latency_us` | Histogram | entity_type, cf | L2 RocksDB |
| `storage_write_latency_us` | Histogram | entity_type, cf | L2 RocksDB |
| `vector_query_latency_us` | Histogram | k | L3 HNSW |
| `fts_query_latency_us` | Histogram | — | L4 Tantivy |
| `hybrid_search_latency_us` | Histogram | — | L3+L4 merge |
| `memory_count` | Gauge | entity_type | L2 RocksDB |
| `storage_size_bytes` | Gauge | cf | L2 RocksDB |
| `compression_ratio` | Gauge | cf | L2 RocksDB |
| `wal_size_bytes` | Gauge | — | RocksDB WAL |
| `engine_py03_call_latency_us` | Histogram | function | PyO3 Bridge |
| `api_latency_ms` | Histogram | method, path | Python FastAPI |
| `active_sessions` | Gauge | — | Python Service |
| `pending_exports` | Gauge | — | Python Service |

### 14.2 Collection Mechanism

- Rust side: `AtomicU64` counters + `Histogram` structs behind `Arc`. Updated on every operation.
- Periodic snapshots (default: 60s interval) flush aggregated metrics to L2 `telemetry` CF.
- Python side: FastAPI middleware records request latencies. Batched and sent to Rust `telemetry` CF.
- UI queries `GET /analytics/health` and `GET /analytics/performance` to display metrics.

### 14.3 Alerting (Future)

- Configurable thresholds per metric (e.g., `cache_hit_ratio < 0.5` → warning)
- Threshold breaches → `Notification` records → in-app notification badge + optional webhook

---

## 15. Edge Cases & Failure Modes

| Scenario | Behavior |
|---|---|
| **Storage path not writable** | Pre-flight check on startup. Error banner in UI, Settings > Storage shows ❌ badge. Read-only mode if RocksDB can be opened read-only. |
| **RocksDB corruption** | Automatic detection on open (MANIFEST checksum failure). Switch to read-only mode, notify user, offer recovery path from last WAL checkpoint. |
| **Vector index out of sync** | Full rebuild triggered on next query if memory count in RocksDB doesn't match HNSW entry count. Iterates all memory_items with embeddings. |
| **Tantivy index stale** | Automatic rebuild from RocksDB memory_items on first query if segment count is 0 but memory count > 0. |
| **WAL replay takes too long** | Progress reported via telemetry. For WAL > 10K unflushed entries, recovery runs in background with a status notification. |
| **Content store GC failure** | Log error, retry on next write. Manual trigger available in Data Management settings. |
| **File watcher misses change** | On startup, all tracked files are scanned and hashed. Missed changes detected on next startup. |
| **Concurrent writes to same entity** | LWW-Register resolves by timestamp. Conflict recorded in conflicts CF. |
| **MCP server restart** | Clients reconnect with exponential backoff. Session state persists in RocksDB. |
| **Export job failure mid-way** | Partial output discarded. Error recorded in export history with retry. |
| **Large memory search (>1M embeddings)** | HNSW handles 1M+ with single-digit ms latency (M=16, ef=50 ~2-5ms). Above 10M, shard by project or use pgvector via StorageBackend trait. |
| **RocksDB disk full** | Catch `StatusCode::IOError` on write. Set storage to read-only. Notify user with disk space warning. |
| **PyO3 bridge panic** | `catch_unwind` at bridge boundary. Convert panic to `PyErr` with message. Rust core stays alive for subsequent calls. |
| **Telemetry CF write amplification** | Telemetry events buffered in memory (DashMap buffer), flushed in batches every 60s. Prevents per-event L2 write. |
| **Cache stampede on cold start** | Cache is empty on startup. First requests hit L2 directly. Cache warms naturally with read traffic. No thundering herd protection needed — L2 is fast enough for single-user. |
