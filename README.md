# Contexter

> Multi-layered context management system for context-aware agents. Manages sessions, memories, agents, skills, and analytics through a three-tier architecture.

Contexter provides persistent, versioned, searchable context for AI agents. It stores agent sessions, conversation memories (with CRDT-based conflict resolution), agent/skill registries, analytics aggregations, and audit trails — all exposed through a REST API, an MCP interface, and a web dashboard.

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│  contexter-web  (React 19 · TypeScript 6 · Vite 8)│
│  Port 5173 — Dashboard UI with 30+ routes        │
└──────────────────────┬───────────────────────────┘
                       │ HTTP /api/v1
┌──────────────────────▼───────────────────────────┐
│  contexter-server  (Python 3.12 · FastAPI)        │
│  Port 8051 — REST API (17 router modules)         │
│  Port 8052 — FastMCP SSE server (8 tools, 4 res.) │
│  12 domain services · Pydantic v2 · structlog     │
└──────────────────────┬───────────────────────────┘
                       │ PyO3 bridge (asyncio→thread)
┌──────────────────────▼───────────────────────────┐
│  contexter-core  (Rust · RocksDB)                 │
│  CRDT state · L1 DashMap+LRU cache · 9 CFs       │
│  Zstd/LZ4 · Tantivy FTS · HNSW vectors · DuckDB  │
└──────────────────────────────────────────────────┘
```

### Layers

| Layer | Directory | Language | Purpose |
|-------|-----------|----------|---------|
| **Core** | `contexter-core/` | Rust (2021) | CRDT-based storage engine, RocksDB backend, caching, compression, FTS, vector index, analytics |
| **Server** | `contexter-server/` | Python 3.12 | FastAPI REST API (port 8051), FastMCP SSE server (port 8052), domain services, bridge to Rust core |
| **Web** | `contexter-web/` | TypeScript 6 | React 19 dashboard UI with 30+ routes, TanStack Query, Tailwind v4 design system |

---

## Quick Start

### Prerequisites

- Rust 1.85+ (`rustup`)
- Python 3.12+
- Node.js 22+
- npm 10+

### 1. Build the Rust core

The Rust engine (`contexter_core`) is a **hard runtime dependency** of the
server. The Python layer does not fall back to stubs or in-memory mocks: if
the engine wheel is not installed, the server refuses to start (see
[Engine as hard dependency](#engine-as-hard-dependency)).

```bash
cd contexter-core
cargo build --release
# Python bindings (requires maturin)
pip install maturin && maturin develop --release
```

### 2. Start the Python API server

```bash
cd contexter-server
pip install -e ".[dev]"
uvicorn contexter_server.main:app --port 8051 --reload
# MCP server starts automatically on port 8052
```

The REST API is available at `http://localhost:8051/api/v1`.
API docs at `http://localhost:8051/docs` (enable with `CONTEXTER_ENABLE_DOCS=true`).

### 3. Start the web dashboard

```bash
cd contexter-web
npm install
npm run dev
```

The dashboard is available at `http://localhost:5173`. It proxies `/api` requests to `http://localhost:8051`.

### 4. Use the CLI

```bash
# Via Rust binary
cargo run --release -- <command>

# Via Python entry point
contexter <command>
```

---

## Configuration

Contexter is configured through environment variables. All variables use the
canonical `CONTEXTER_` prefix.

| Variable | Applies to | Default | Purpose |
|----------|------------|---------|---------|
| `CONTEXTER_API_KEY` | server (REST + MCP) | unset | API-key authentication. When set, REST requests must present `Authorization: Bearer <key>`, and MCP tool calls and resource reads must supply a matching `_api_key` (tool parameter, or query parameter on resource URIs). When unset, API-key authentication is disabled. |
| `CONTEXTER_BRIDGE_POOL_SIZE` | server | `8` | Worker count for the bounded thread pool that offloads Rust engine calls. Invalid or non-positive values fall back to the default. |
| `CONTEXTER_ENABLE_DOCS` | server | unset | Set to `true` to enable the interactive OpenAPI docs at `http://localhost:8051/docs`. |
| `CONTEXTER_MAX_REQUEST_BODY` | server (REST) | `1048576` | Maximum accepted REST request body size in bytes. Requests with a larger `Content-Length` (or chunked transfer encoding) are rejected with HTTP 413. |
| `CONTEXTER_PATH` | CLI (Python) | unset | Data directory used by the `contexter` CLI entry point. |
| `CONTEXTER_DB_PATH` | CLI (Rust) | unset | RocksDB data path used by the `contexter-core` binary. |

---

## MCP Interface (SSE)

The MCP server runs with **SSE (server-sent events) transport** on port
**8052** (`mcp.run(transport="sse", port=_MCP_PORT)` in `main.py`) and
exposes 8 tools and 4 read-only resources. When
`CONTEXTER_API_KEY` is set, every MCP tool call and resource read must
supply a matching `_api_key` value:

- **Tools** — `_api_key` is a tool argument.
- **Resources** — `_api_key` is an optional query parameter on the resource
  URI (RFC 6570 `{?_api_key}` suffix).

Keys are compared constant-time (`hmac.compare_digest`). A missing or
mismatched key raises `MCPAuthError`, which FastMCP serialises as a clean
JSON-RPC error rather than an internal server fault. When `CONTEXTER_API_KEY`
is unset, the check is skipped (backward-compatible development mode).

### Read-only resources

| URI | Description |
|-----|-------------|
| `contexter://session/{id}{?_api_key}` | Session data as JSON |
| `contexter://memory/{id}{?_api_key}` | Memory data as JSON |
| `contexter://agent/{id}{?_api_key}` | Agent data as JSON |
| `contexter://analytics/overview{?_api_key}` | Analytics overview as JSON |

---

## Project Structure

```
contexter/
├── contexter-core/          # Rust storage engine
│   ├── src/
│   │   ├── engine/          # Unified API (session, memory, agent, skill, search, analytics)
│   │   ├── storage/         # RocksDB backend, column families, migrations
│   │   ├── models/          # Domain entities (DDD per-type files)
│   │   ├── cache/           # L1 DashMap + LRU hot cache
│   │   ├── crdt/            # CRDT conflict resolution (LWW-Register)
│   │   ├── compression/     # Zstd / LZ4 / Noop codecs
│   │   ├── versioning/      # Content-addressed version store + GC
│   │   ├── vector/          # HNSW vector index
│   │   ├── fts/             # Tantivy full-text search
│   │   ├── analytics/       # DuckDB analytics aggregation
│   │   ├── telemetry/       # Self-observability (metrics, tracing)
│   │   ├── wal/             # Write-ahead log
│   │   └── bridge.rs        # PyO3 FFI bridge
│   └── tests/               # 20+ integration test modules
│
├── contexter-server/        # Python API layer
│   ├── src/contexter_server/
│   │   ├── api/             # 17 FastAPI router modules (REST endpoints)
│   │   ├── services/        # 12 domain service classes
│   │   ├── models/          # Pydantic v2 schemas
│   │   ├── core/bridge.py   # Async StorageEngine wrapper
│   │   ├── mcp_tools/       # FastMCP tool/resource handlers
│   │   └── cli/             # Click CLI commands
│   └── tests/               # Pytest suite (api, services, models, mcp, cli)
│
├── contexter-web/           # React dashboard
│   ├── src/
│   │   ├── api/             # Client, types, 18 React Query hook modules
│   │   ├── pages/           # 17 page directories (30+ routes)
│   │   ├── components/      # AppShell layout + 20 shared UI primitives
│   │   └── styles/          # V2-DEEP design tokens (Tailwind v4)
│   └── tests/               # Vitest + MSW mocks + test factories
│
├── docs/                    # Architecture, contracts, ADRs
└── Cargo.toml               # Workspace root
```

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| **Core** | Rust 2021, RocksDB, PyO3, Tantivy, HNSW (instant-distance), DuckDB, DashMap, LRU, Zstd/LZ4, Serde, UUID v7 |
| **Server** | Python 3.12, FastAPI, FastMCP, Pydantic v2, structlog, slowapi, Click, uvicorn, httpx |
| **Web** | React 19, TypeScript 6 (strict), Vite 8, Tailwind v4, React Router v7, TanStack Query v5, Framer Motion, Recharts, Lucide, Vitest, MSW |

---

## Key Features

- **CRDT-based state** — Last-Writer-Wins Register conflict resolution for concurrent session/memory updates
- **Multi-tier storage** — L1 DashMap+LRU hot cache, L2 RocksDB with 9 column families, configurable compression
- **Versioning** — Content-addressed version store with diff computation and garbage collection
- **Full-text search** — Tantivy-indexed memory and session search with faceted filtering
- **Vector search** — HNSW index for similarity-based memory retrieval
- **Analytics** — DuckDB-powered aggregation for efficiency, cost, and performance metrics
- **Agent/skill registry** — Lifecycle management with capability and effectiveness tracking
- **Audit trail** — Immutable, queryable audit log for all entity mutations
- **Export** — JSON/CSV export of sessions, memories, and analytics
- **MCP interface** — 8 tools + 4 resources for LLM integration via FastMCP SSE

---

## Design Decisions

The following architecture decisions are accepted and intentional. They are
documented here so future changes are evaluated against the original rationale
rather than revisited blindly.

### Engine as hard dependency

`contexter_core` (the Rust PyO3 wheel) is a **hard runtime dependency** of
`contexter-server`. The server import guard raises `ImportError` when the
wheel is missing, and the bridge **refuses to dispatch mocked engine
methods** at runtime: if an engine method resolves to a `unittest.mock`
object, the call raises `TypeError` instead of silently returning mock data.
There is no mock fallback for production operation. This guarantees that
analytics, telemetry, and storage numbers always come from the real engine.

### Bounded thread-pool bridge

All Rust engine calls are offloaded from the async event loop via
`loop.run_in_executor()` on a **bounded, explicitly managed**
`ThreadPoolExecutor` (default 8 workers, configurable with
`CONTEXTER_BRIDGE_POOL_SIZE`). This is a deliberate decision over bare
`asyncio.to_thread()`:

- `asyncio.to_thread()` runs on the loop's *default* executor, whose thread
  count is not under application control — under load, concurrent engine
  calls can drive unbounded thread growth.
- A bounded pool caps concurrent engine calls at a known maximum, making
  thread usage and RocksDB contention predictable and tunable.
- The pool is created once per `StorageEngine` instance and reused for the
  lifetime of the process.

JSON serialisation/deserialisation happens at the boundary; large memory
content (≥ 100 KB) is passed as raw `PyBytes` to avoid double-encoding
overhead.

### Memory content is stored lowercased (REQ-S-003)

The Rust engine **pre-lowercases memory content on write** for performant
keyword search (`content.to_lowercase()` in both `create_memory` and
`update_memory`, see `contexter-core/src/storage/rocksdb.rs`). Content read
back from the engine is therefore lowercased — a 102,400-byte round-trip
returning lowercased content is **expected behavior, not a bug**. Bridge
byte-identity guarantees apply to case-stable content (lowercase ASCII or
CJK); for mixed-case input, expect the stored form to be lowercase.

### Telemetry mapping

The Rust engine emits distinct telemetry shapes per call — there is no
single casing contract at the FFI boundary:

- `cache_telemetry()` → **snake_case**: `gets`, `hits`, `misses`, `stores`,
  `invalidations`, `total_ops`, `entries_by_type`
- `storage_size()` → **camelCase**: `total`, `perCf`, `walSize`
- `status()` → **camelCase**: `status`, `version`, plus a nested
  `cacheTelemetry` object (`hits`, `misses`, `totalOps`, `hitRatio`,
  `entriesByType`)

The analytics layer (`AnalyticsService`) is the anti-corruption layer that
maps these engine shapes onto the analytics domain models — snake_case
fields such as `total_sessions`, `storage_size_bytes`, `total_operations`,
and `cache_hit_rate`. Every read is explicit and key mismatches are logged,
never silently defaulted. `_safe_get` only guards against non-dict results
(exceptions, `None`) — it **no longer masks key mismatches**: telemetry keys
must map correctly and are covered by tests, so counters reflect real engine
data instead of structurally defaulting to zero.

### Accepted performance decisions

The following performance characteristics are accepted by design and are
covered by the MCP performance contract (PF-05..PF-11). They are documented
here so future changes are evaluated against the original rationale:

- **Per-call logs at DEBUG, not INFO** — `bridge_call_end` (bridge) and
  `call_received` / `auth_decision` / `engine_result` (MCP handlers) fire
  once per call and are logged at **DEBUG**, so the default INFO level stays
  quiet under sustained MCP call rates. INFO is reserved for lifecycle and
  error events; the failure path logs at ERROR with bounded context (no
  content payloads or secrets, per the observability contract). Enable DEBUG
  only when tracing per-call behaviour is needed.
- **MCP list tools bounded at 100, no pagination** — `list_skills` has no
  limit parameter (frozen MCP contract; engine default 100) and
  `list_recent_sessions` defaults to the engine's 100-entry cap. Explicit
  session limits are clamped to `MAX_SESSION_LIST_LIMIT` (10,000) and pushed
  down to the engine — no Python re-slicing. Pagination is deferred until the
  MCP contract is next revised.
- **`store_memory` makes exactly two sequential engine calls** — a session
  lookup (`get_session`) followed by `create_memory`, because the memory's
  `agent_id` is derived from the session. This is deliberate, not an N+1
  pattern; the calls are sequential because the second depends on the first.
- **`export_data` reads up to 10,000 records per entity into memory** —
  bounded, never unbounded, and results are cached in an in-memory LRU
  (max 100 entries) with persistence through `set_setting` / `get_setting`,
  so repeated exports of the same scope do not re-read the engine.
- **Unfiltered counts are `estimate-num-keys` estimates, not exact
  scans** — `count_sessions`, `count_agents`, and `count_skills` (and
  `count_memories`) return RocksDB's `rocksdb.estimate-num-keys` property
  when called without a filter: an O(1) lookup, not a full scan. The
  estimate is exact on a freshly seeded store, but it counts **memtable
  update history** — every update or delete leaves a newer key version,
  so the estimate inflates until background compaction merges those
  versions. `flush()` does **not** correct it: it only writes the memtable
  to an SST that still holds multiple versions of the same keys. Measured
  behaviour: 100 creates → 100/100 (exact); +100 updates → 200 vs 100
  actual (2×); +50 deletes → 150 vs 50 (3×); after `flush()` → still 170
  vs 60. `get_overview` and the CLI `contexter status` surface the estimate
  directly (measured 210 vs 100 actual after session creates plus
  turn-count updates), and sessions are the highest-mutation-frequency
  entity, so the inflation is most visible there. Exact counts remain
  available via **filtered counts** (e.g. `count_sessions({"project":
  ...})`, which index-prefix scans) or the `list_sessions` /
  `list_agents` / `list_skills` tools — with the tradeoff that the list
  tools are bounded at 100 entries (no pagination, see above), so
  filtered counts are the exactness path for larger datasets. Automatic
  correction happens only through RocksDB's background compaction; no
  compaction trigger is exposed today (`contexter gc` runs flush +
  checkpoint, which does not correct the estimate).

---

## Sub-project READMEs

- [`contexter-core/`](contexter-core/README.md) — Rust storage engine documentation
- [`contexter-server/`](contexter-server/README.md) — Python API server documentation
- [`contexter-web/`](contexter-web/README.md) — React dashboard documentation

---

## License

MIT
