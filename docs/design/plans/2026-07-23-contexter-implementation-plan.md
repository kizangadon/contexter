# Contexter — Full Implementation Plan

> **For agentic workers:** Tasks use checkbox (`- [ ]`) syntax. Each phase is an independent deliverable. Validation contracts (`SPEC.md`, `ACCEPTANCE.md`, `EDGE_CASES.md`) per phase contain the detailed technical specs.

**Goal:** Build Contexter — a RAG-like memory, agent, skill, and session management platform for AI coding agents. Replaces rekal entirely. Exposes both REST API (FastAPI) and MCP Server (FastMCP) over a Rust+Python modular monolith with a React SPA frontend.

**Architecture:** Three-layer modular monolith — Rust core (RocksDB multi-tier storage, HNSW vector, Tantivy FTS, DuckDB analytics) exposed via PyO3 to a Python management layer (FastAPI + FastMCP + services), consumed by a React SPA (Tailwind v4, React Query, React Router v7). Single process, zero network calls between Rust and Python.

**Tech Stack:** Rust (rust-rocksdb, instant-distance/voyager, tantivy, duckdb, pyo3), Python (fastapi, fastmcp, pydantic-v2, click), React 19 + TypeScript (Tailwind v4, TanStack Query, React Router v7, Recharts, Framer Motion)

**Spec Hub:** `docs/design/specs/2026-07-23-contexter-specification-hub.md`
**Architecture Spec:** `docs/design/specs/2026-07-23-contexter-system-architecture.md`
**UI Design Specs:** `docs/design/specs/2026-07-23-contexter-ui-design.md` and sub-specs

---

## Phase 1: Rust Core Foundation

**Goal:** Build the Rust core engine with RocksDB multi-tier storage, the PyO3 bridge, and Session + Memory CRUD operations, verified through a working CLI tool.

**Dependencies:** None (this is the foundation)

**Key deliverables:**
- `contexter-core/` — Rust crate with module tree, `StorageBackend` trait, RocksDB implementation (8 column families), `Engine` struct composing all tiers, `#[pyclass]` bridge
- `core_bridge.py` — Python async wrapper with `asyncio.to_thread()` + `ThreadPoolExecutor`
- `cli/main.py` — Click CLI for diagnostics (session CRUD, memory CRUD, status)
- Test suite: inline `#[cfg(test)]` per source file + integration tests in `tests/`

### Task 1.1: Rust project skeleton

Initialize the Rust crate with workspace, module tree, and all key dependencies. Every module gets a scaffold with inline `#[cfg(test)] mod tests { }`.

**Files:**
- `contexter-core/Cargo.toml`
- `contexter-core/src/lib.rs`
- `contexter-core/src/bridge.rs`
- `contexter-core/src/engine/mod.rs`
- `contexter-core/src/storage/mod.rs`
- `contexter-core/src/cache/mod.rs`
- `contexter-core/src/vector/mod.rs`
- `contexter-core/src/fts/mod.rs`
- `contexter-core/src/analytics/mod.rs`
- `contexter-core/src/compression/mod.rs`
- `contexter-core/src/crdt/mod.rs`
- `contexter-core/src/versioning/mod.rs`
- `contexter-core/src/models/mod.rs`
- `contexter-core/src/models/memory.rs`, `session.rs`, `agent.rs`, `skill.rs`, `settings.rs`, `audit.rs`, `telemetry.rs`, `notification.rs`, `feedback.rs`, `correlation.rs`, `analytics.rs`
- `contexter-core/src/telemetry/mod.rs`
- `contexter-core/src/util/mod.rs`
- `contexter-core/src/wal/mod.rs`

### Task 1.2: Data models (Rust)

Implement all entity structs with `serde` Serialize/Deserialize, UUID v7 primary keys, and chrono UTC timestamps.

**Files:**
- `contexter-core/src/models/*.rs` (all entity files above)

### Task 1.3: RocksDB storage layer

`StorageBackend` trait + `RocksDbBackend` implementation with 8 column families, per-CF compression (Zstd/LZ4), key encoding, and Session + Memory CRUD.

**Files:**
- `contexter-core/src/storage/mod.rs` — trait
- `contexter-core/src/storage/rocksdb.rs` — implementation
- `contexter-core/src/storage/column_families.rs` — CF definitions, key encoding, per-CF config
- `contexter-core/src/storage/migrations.rs` — schema version tracking
- `contexter-core/src/storage/types.rs` — row↔struct conversion

### Task 1.4: L1 Hot Cache (DashMap + LRU)

Write-through/write-around cache for frequently accessed entities.

**Files:**
- `contexter-core/src/cache/mod.rs` — Cache trait
- `contexter-core/src/cache/dashmap_lru.rs` — implementation
- `contexter-core/src/cache/metrics.rs` — hit/miss counters

### Task 1.5: Engine struct

Top-level `Engine` composing cache + storage (stubs for L3–L5). Provides session and memory CRUD operations.

**Files:**
- `contexter-core/src/engine/mod.rs`
- `contexter-core/src/engine/session.rs`
- `contexter-core/src/engine/memory.rs`
- `contexter-core/src/engine/search.rs`
- `contexter-core/src/engine/export.rs`
- `contexter-core/src/engine/analytics.rs`

### Task 1.6: PyO3 bridge

Single `#[pyclass] Engine` exposing all session/memory operations plus generic `store(cf, key, value)` / `get(cf, key)`.

**Files:**
- `contexter-core/src/bridge.rs` — all `#[pyfunction]` and `#[pyclass]` declarations
- Python side: `contexter-server/src/core/bridge.py` — async wrapper

### Task 1.7: CLI tool

Click-based CLI for diagnostics: `contexter session create|list|get|delete`, `contexter memory create|search`, `contexter status`.

**Files:**
- `contexter-server/src/cli/main.py`
- `contexter-server/pyproject.toml` (or setup.py)

### Task 1.8: Compression utilities

Zstd and LZ4 wrappers for use across the codebase.

**Files:**
- `contexter-core/src/compression/mod.rs`
- `contexter-core/src/compression/codecs.rs`

### Task 1.9: Test suite

Complete inline unit tests + integration tests in `tests/` mirroring `src/`.

**Files:**
- `contexter-core/tests/storage/rocksdb_test.rs`
- `contexter-core/tests/cache/lru_test.rs`
- `contexter-core/tests/engine/session_test.rs`
- `contexter-core/tests/engine/memory_test.rs`
- `contexter-core/tests/bridges/pyo3_test.rs`
- `contexter-core/tests/compression/codecs_test.rs`
- `contexter-core/tests/common/mod.rs` (test helpers)

### Phase 1 Checkpoints

```
Checkpoint A (after 1.3):  cargo test passes, temp RocksDB CRUD verified
Checkpoint B (after 1.6):  Python script creates/reads sessions via PyO3 bridge
Checkpoint C (after 1.9):  All tests pass, clippy clean, human review
```

---

## Phase 2: Search & Analytics Engine

**Goal:** Add vector search (L3 HNSW), full-text search (L4 Tantivy), and analytical queries (L5 DuckDB) to the Rust core. Implement hybrid search merging L3 + L4 results.

**Dependencies:** Phase 1 complete (RocksDB storage + Engine struct exist)

**Key deliverables:**
- HNSW vector index with binary snapshot persistence
- Tantivy full-text index with incremental indexing
- DuckDB analytics engine with on-demand sync from RocksDB
- Hybrid search merge + rerank
- Efficiency & correlation computation

### Task 2.1: HNSW vector index (L3)

Implement HNSW graph for ANN search with cosine similarity, binary snapshot persistence, and auto-rebuild on mismatch.

**Files:**
- `contexter-core/src/vector/mod.rs` — VectorIndex trait
- `contexter-core/src/vector/hnsw.rs` — HNSW implementation
- `contexter-core/src/vector/distance.rs` — distance kernels
- `contexter-core/src/vector/snapshot.rs` — persist/load

### Task 2.2: Tantivy full-text search (L4)

Implement Tantivy index with entity-type-specific schemas, field boosting, incremental indexing, and query parsing.

**Files:**
- `contexter-core/src/fts/mod.rs` — FullTextIndex trait
- `contexter-core/src/fts/tantivy.rs` — Tantivy wrapper
- `contexter-core/src/fts/schema.rs` — index schemas
- `contexter-core/src/fts/query.rs` — query parsing

### Task 2.3: DuckDB analytics engine (L5)

Implement DuckDB integration for columnar analytical queries with on-demand sync from RocksDB.

**Files:**
- `contexter-core/src/analytics/mod.rs` — AnalyticsEngine trait
- `contexter-core/src/analytics/duckdb.rs` — DuckDB wrapper
- `contexter-core/src/analytics/queries.rs` — predefined SQL
- `contexter-core/src/analytics/sync.rs` — RocksDB→DuckDB sync

### Task 2.4: Hybrid search

Merge L3 (HNSW) + L4 (Tantivy) results with configurable weighting, reranking, and filter application.

**Files:**
- `contexter-core/src/engine/search.rs` (update with hybrid path)

### Task 2.5: Efficiency & correlation computation

Implement session efficiency calculation and metric correlation (Pearson/Spearman) using DuckDB.

**Files:**
- `contexter-core/src/engine/analytics.rs` (update)

### Task 2.6: L3–L5 test suite

Inline unit tests + integration tests for all new modules.

**Files:**
- `contexter-core/tests/vector/hnsw_test.rs`
- `contexter-core/tests/fts/tantivy_test.rs`
- `contexter-core/tests/analytics/duckdb_test.rs`

### Phase 2 Checkpoints

```
Checkpoint A (after 2.1):  HNSW stores/retrieves vectors, snapshot round-trips
Checkpoint B (after 2.2):  Tantivy indexes and searches content
Checkpoint C (after 2.4):  Hybrid search returns merged results
Checkpoint D (after 2.6):  All tests pass, clippy clean
```

---

## Phase 3: Python API Layer

**Goal:** Build the Python management layer — FastAPI REST server (port 8000), FastMCP server (port 8001), and all service/orchestration logic on top of the Rust engine via PyO3.

**Dependencies:** Phase 1 complete (PyO3 bridge with session/memory CRUD)

**Key deliverables:**
- FastAPI application with all route modules (15+ endpoint groups)
- FastMCP server with tools and resources
- Python service layer (session, memory, agent, skill, analytics, export, audit, etc.)
- Pydantic model schemas mirroring Rust types
- Settings management from `config.yaml`
- Complete Python test suite

### Task 3.1: Python project skeleton

Initialize project structure with `pyproject.toml`, all dependencies, module tree mirroring the architecture spec.

**Files:**
- `contexter-server/pyproject.toml`
- `contexter-server/src/main.py`
- `contexter-server/src/mcp_server.py`
- `contexter-server/src/__init__.py`

### Task 3.2: Pydantic models

Implement all Pydantic v2 models matching Rust entities.

**Files:**
- `contexter-server/src/models/*.py`

### Task 3.3: Core bridge wrapper

Enhance `core_bridge.py` with full session/memory/agent/skill/analytics coverage.

**Files:**
- `contexter-server/src/core/bridge.py`

### Task 3.4: Service layer

Implement all service modules with business logic orchestration.

**Files:**
- `contexter-server/src/services/*.py`

### Task 3.5: FastAPI REST server

Implement all API route modules under `/api/v1/`.

**Files:**
- `contexter-server/src/api/*.py`

### Task 3.6: FastMCP server

Implement all MCP tools and resources.

**Files:**
- `contexter-server/src/mcp_tools/*.py`
- `contexter-server/src/mcp_server.py` (update)

### Task 3.7: Settings & config management

Read/write `config.yaml`, validate paths, manage data directory.

**Files:**
- `contexter-server/src/services/settings_service.py`

### Task 3.8: Python test suite

Complete service tests (mocked bridge), API tests (TestClient), MCP tests, model tests, CLI tests.

**Files:**
- `contexter-server/tests/conftest.py`
- `contexter-server/tests/services/*.py`
- `contexter-server/tests/api/*.py`
- `contexter-server/tests/mcp/*.py`
- `contexter-server/tests/models/*.py`
- `contexter-server/tests/cli/*.py`

### Phase 3 Checkpoints

```
Checkpoint A (after 3.5):  REST API responds on port 8000, CRUD works via curl
Checkpoint B (after 3.6):  MCP tools respond on port 8001
Checkpoint C (after 3.8):  All Python tests pass
```

---

## Phase 4: React UI

**Goal:** Build the full React SPA with all 22+ pages, the V2-DEEP design system, and complete feature coverage.

**Dependencies:** Phase 3 complete (REST API exists and is testable)

**Key deliverables:**
- React 19 + TypeScript project with Vite
- V2-DEEP design system tokens and component library
- AppShell with sidebar navigation
- All pages: Dashboard, Session Manager + Detail, Memory Explorer + Detail, Agent Registry + Detail, Skill Registry + Detail, Efficiency Mapper + 6 detail pages, Analytics + 6 detail pages, Settings (8 sections), Notifications, Feedback, Onboarding, API Playground, Search, Exports, Correlation, Audit
- React Query hooks, MSW test setup, complete test suite

### Task 4.1: React project skeleton

Initialize with Vite + React 19 + TypeScript + Tailwind v4. Configure React Router v7, TanStack Query, Framer Motion.

**Files:**
- `contexter-web/package.json`
- `contexter-web/vite.config.ts`
- `contexter-web/tailwind.config.ts`
- `contexter-web/tsconfig.json`

### Task 4.2: Design system implementation

V2-DEEP tokens as CSS variables, shared component library (StatCard, DataTable, Tag, FilterBar, TimeframeSelector, Modal, Toast, EmptyState, LoadingSkeleton).

**Files:**
- `contexter-web/src/styles/tokens.css`
- `contexter-web/src/components/shared/*.tsx`
- `contexter-web/src/components/common/*.tsx`

### Task 4.3: AppShell + navigation

Sidebar nav, top bar, breadcrumbs, responsive layout.

**Files:**
- `contexter-web/src/App.tsx`
- `contexter-web/src/components/layout/*.tsx`

### Task 4.4: API client + hooks

Auto-generated API client + React Query hooks per domain.

**Files:**
- `contexter-web/src/api/client.ts`
- `contexter-web/src/api/hooks/*.ts`

### Task 4.5: Phase 2 core UI pages

Dashboard, Session Manager, Session Detail, Memory Explorer, Memory Detail, Agent Registry, Agent Detail, Skill Registry, Skill Detail, Efficiency Mapper.

**Files:**
- `contexter-web/src/pages/Dashboard/*.tsx`
- `contexter-web/src/pages/Sessions/*.tsx`
- `contexter-web/src/pages/Memories/*.tsx`
- `contexter-web/src/pages/Agents/*.tsx`
- `contexter-web/src/pages/Skills/*.tsx`
- `contexter-web/src/pages/Efficiency/*.tsx`

### Task 4.6: Analytics pages

Analytics Overview, System Health, Performance Trends, Resource Usage, Cost & Token Analytics, Model Detail, Service Status.

**Files:**
- `contexter-web/src/pages/Analytics/*.tsx`

### Task 4.7: Settings pages

8 settings sections with sidebar layout.

**Files:**
- `contexter-web/src/pages/Settings/*.tsx`

### Task 4.8: Standalone feature pages

Notifications, Feedback (3 tabs), Onboarding (wizard), API Playground (REST/MCP tabs), Search, Exports (3 tabs), Correlation (3 tabs), Audit (3 tabs).

**Files:**
- `contexter-web/src/pages/Notifications/*.tsx`
- `contexter-web/src/pages/Feedback/*.tsx`
- `contexter-web/src/pages/Onboarding/*.tsx`
- `contexter-web/src/pages/Playground/*.tsx`
- `contexter-web/src/pages/Search/*.tsx`
- `contexter-web/src/pages/Exports/*.tsx`
- `contexter-web/src/pages/Correlation/*.tsx`
- `contexter-web/src/pages/Audit/*.tsx`

### Task 4.9: Test suite

Component tests, hook tests, API client tests, route/page tests with MSW.

**Files:**
- `contexter-web/tests/setup.ts`
- `contexter-web/tests/components/**/*.test.tsx`
- `contexter-web/tests/hooks/*.test.ts`
- `contexter-web/tests/api/*.test.ts`
- `contexter-web/tests/routes/*.test.tsx`

### Phase 4 Checkpoints

```
Checkpoint A (after 4.3):  AppShell renders, navigation works
Checkpoint B (after 4.5):  All 10 core pages render with live data
Checkpoint C (after 4.6-4.7):  Analytics + Settings pages render
Checkpoint D (after 4.8):  All standalone features render
Checkpoint E (after 4.9):  All UI tests pass
```

---

## Risk Register

| Risk | Impact | Mitigation |
|---|---|---|
| Rust learning curve for Python devs | Med | Core is written once. Python devs only touch the bridge interface. |
| RocksDB write amplification under load | Low | Per-CF compression tuning. Telemetry uses buffered batch writes, not per-event. |
| PyO3 GIL contention with 4 workers | Low | `asyncio.to_thread` + dedicated thread pool. RocksDB ops are microsecond-level. |
| HNSW recall degradation at scale | Med | Auto-rebuild on count mismatch. Configurable ef_search for accuracy/speed trade-off. |
| DuckDB memory pressure on large analytics | Low | Analytics queries bounded by timeframe. Retention: 90 days default. |
| React UI scope (22+ pages) | High | Phase 4 is the largest. Sub-phases are independently shippable. |

---

## Undelivered Priorities

- **Phase 2 (Search & Analytics):** Can be deferred if MVP only needs CRUD + basic search. L4 Tantivy + hybrid search can ship as Phase 2.1, L5 DuckDB as Phase 2.2.
- **Phase 4 standalone features:** Notifications, Feedback, Onboarding, API Playground, Correlation, Audit are lower priority than core pages + analytics + settings.
- **Plugin storage backend (PostgreSQL/pgvector):** Not in scope for initial build. `StorageBackend` trait exists for future implementation.
- **Multi-user auth:** Not in scope for initial build. JWT auth structure exists but user management is deferred.
