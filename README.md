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
API docs at `http://localhost:8051/docs` (enable with `CONtexTER_ENABLE_DOCS=true`).

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

## Sub-project READMEs

- [`contexter-core/`](contexter-core/README.md) — Rust storage engine documentation
- [`contexter-server/`](contexter-server/README.md) — Python API server documentation
- [`contexter-web/`](contexter-web/README.md) — React dashboard documentation

---

## License

MIT
