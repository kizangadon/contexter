---
title: "Phase 3 — Python API Layer"
version: 1.0
date_created: 2026-07-25
owner: Contexter Team
tags: python, fastapi, fastmcp, pydantic, pyo3, api
---

# Phase 3 — Python API Layer

## 1. Purpose & Scope

Build the Python management layer for Contexter — a FastAPI REST server (port 8051), a FastMCP server (port 8052), and all service/orchestration logic on top of the Rust core engine via PyO3. The Python layer is the bridge between the Rust storage engine and external consumers (React UI, AI agents via MCP, CLI admin tools).

**Audience:** Implementation engineers, validators, AI coding agents.

**Assumptions:**
- Rust core (`contexter-core`) is compiled with `feature = "python"` and importable as `contexter_core`
- Ports 8051 (REST) and 8052 (MCP) are available
- The data directory is `~/.contexter/`
- All Rust entities and engine methods are exposed via the PyO3 bridge

## 2. Definitions

| Term | Definition |
|---|---|
| Bridge | Python wrapper around the Rust `Engine` PyO3 class, providing sync + async access |
| Service | Python module containing business logic orchestration for a domain concept |
| MCP | Model Context Protocol — an AI agent tool protocol |
| Ubiquitous Language | DDD concept: a shared, rigorous language between developers and domain experts reflected in code names |
| Aggregate | DDD concept: cluster of domain objects treated as a single unit with a root entity |

## 3. Requirements, Constraints & Guidelines

### Build & Project Structure

- **REQ-BLD-001**: `contexter-server/` SHALL be a Python project with `pyproject.toml` declaring all dependencies
- **REQ-BLD-002**: `contexter-core/pyproject.toml` SHALL exist for maturin-based builds, producing the importable `contexter_core` Python module
- **REQ-BLD-003**: The build workflow SHALL be: `maturin develop --release -m contexter-core/pyproject.toml` from the project root
- **REQ-BLD-004**: Module tree SHALL mirror `contexter-server/src/{api,services,models,core,mcp_tools,cli}/` with parallel `tests/` directory

### Domain-Driven Design

- **REQ-DDD-001**: All class names, method names, parameter names, and module names SHALL use ubiquitous language from the Contexter domain
- **REQ-DDD-002**: Module boundaries SHALL follow bounded contexts: sessions, memories, agents, skills, analytics, settings, audit, notifications, export, correlation, search
- **REQ-DDD-003**: Business logic SHALL reside in service modules (not in API route handlers or Pydantic models)
- **REQ-DDD-004**: Service methods SHALL operate on domain objects, not raw dicts or JSON
- **REQ-DDD-005**: The bridge wrapper SHALL be audited and refactored to align with DDD naming conventions

### Test-Driven Development

- **REQ-TDD-001**: Every implementation file SHALL have a corresponding test file
- **REQ-TDD-002**: Tests SHALL be written before implementation (red-green-refactor)
- **REQ-TDD-003**: The bridge module SHALL have tests covering: all CRUD operations, error propagation, large content paths (>100KB), thread pool behavior
- **REQ-TDD-004**: Service tests SHALL use a mocked `StorageEngine` (Python mock of the Rust bridge)
- **REQ-TDD-005**: API tests SHALL use FastAPI `TestClient` with dependency overrides
- **REQ-TDD-006**: MCP tests SHALL use an MCP client test harness
- **REQ-TDD-007**: Model tests SHALL cover: field validation, type coercion, serialization round-trips, edge cases per EDGE_CASES.md

### Core Bridge

- **REQ-BRG-001**: `contexter-server/src/core/bridge.py` SHALL wrap the Rust `contexter_core.Engine` class
- **REQ-BRG-002**: The import SHALL be `from contexter_core import Engine` (not `contexter`)
- **REQ-BRG-003**: All Rust calls SHALL use `asyncio.to_thread()` with a `ThreadPoolExecutor(max_workers=4)` to avoid blocking the event loop
- **REQ-BRG-004**: JSON serialisation/deserialisation SHALL happen at the bridge boundary (Python dict → json.dumps → Rust → json.loads → Python dict)
- **REQ-BRG-005**: Large memory content (>100KB) SHALL use a direct PyBytes path to avoid double JSON encoding overhead
- **REQ-BRG-006**: Bridge methods SHALL return `Optional[dict]` for get operations (None = not found), `list[dict]` for list operations, `dict` for create/update, `None` for delete
- **REQ-BRG-007**: Errors from the Rust engine SHALL be propagated as Python exceptions, not silently swallowed

### Services

- **REQ-SVC-001**: A service module SHALL exist for each bounded context: session, memory, agent, skill, analytics, search, export, notification, audit, correlation, onboarding, settings
- **REQ-SVC-002**: Each service SHALL accept a `StorageEngine` instance via constructor injection
- **REQ-SVC-003**: Services SHALL contain business logic (validation rules, computed fields, cross-entity coordination)
- **REQ-SVC-004**: Services SHALL NOT depend on FastAPI or any HTTP framework

### REST API (FastAPI — port 8051)

- **REQ-API-001**: FastAPI application SHALL listen on port 8051
- **REQ-API-002**: All endpoints SHALL be under `/api/v1/`
- **REQ-API-003**: OpenAPI spec SHALL be auto-generated by FastAPI
- **REQ-API-004**: Endpoint groups SHALL match the architecture spec Section 11.1:
  - Sessions: `GET/POST /sessions`, `GET/PUT/DELETE /sessions/:id`, `POST /sessions/:id/resume`
  - Memories: `GET/POST /memories`, `GET/PUT/DELETE /memories/:id`, `GET /memories/search`, `POST /memories/:id/versions`
  - Agents: `GET/POST /agents`, `GET/PUT/DELETE /agents/:id`
  - Skills: `GET/POST /skills`, `GET/PUT/DELETE /skills/:id`
  - Analytics: `GET /analytics/overview`, `GET /analytics/health`, `GET /analytics/performance`, `GET /analytics/resources`, `GET /analytics/costs`, `GET /analytics/costs/models/:id`, `GET /analytics/services`
  - Efficiency: `GET /efficiency/overview`, `GET /efficiency/memory`, `GET /efficiency/sessions`, `GET /efficiency/agents`, `GET /efficiency/skills`, `GET /efficiency/tokens`, `GET /efficiency/correlation`
  - Search: `GET /search?q=&type=&project=&page=&limit=`
  - Settings: `GET/PUT /settings/:section`
  - Notifications: `GET /notifications`, `PUT /notifications/:id/read`, `POST /notifications/read-all`
  - Audit: `GET /audit?entity_type=&action=&actor=&q=&limit=&offset=`
  - Files: `GET /files?path=`, `GET /files/:hash/diff?base=&compare=`, `POST /files/watch`
  - Correlation: `GET /correlation/overview?timeframe=`, `GET /correlation/timeline?project=&agent=`, `GET /correlation/compare?a=&b=`
  - Export: `POST /export/submit`, `GET /export/status/:id`, `GET /export/download/:id`, `GET /export/history`
  - Feedback: `POST /feedback/bug`, `POST /feedback/suggest`, `GET /changelog`
  - Onboarding: `GET /onboarding/status`, `POST /onboarding/wizard`, `GET /onboarding/progress`
- **REQ-API-005**: Route handlers SHALL delegate to service layer — no business logic in route handlers
- **REQ-API-006**: 404 SHALL be returned for entity not found, 422 for validation errors, 500 for internal errors

### MCP Server (FastMCP — port 8052)

- **REQ-MCP-001**: FastMCP server SHALL listen on port 8052
- **REQ-MCP-002**: Transport SHALL be SSE (Server-Sent Events)
- **REQ-MCP-003**: Tools SHALL include: `store_memory`, `search_memories`, `get_session`, `list_recent_sessions`, `get_agent_info`, `list_skills`, `get_system_health`, `export_data`
- **REQ-MCP-004**: Resources SHALL include: `contexter://session/{id}`, `contexter://memory/{id}`, `contexter://agent/{id}`, `contexter://analytics/overview`

### Settings & Configuration

- **REQ-CFG-001**: Settings SHALL be read from `~/.contexter/config.yaml`
- **REQ-CFG-002**: The config file SHALL be created with defaults if it does not exist
- **REQ-CFG-003**: Settings sections SHALL mirror architecture spec Section 12.2: project, storage, cache, mcp_server, llm_providers, notifications, versioning, analytics, telemetry
- **REQ-CFG-004**: Port configuration for REST (8051) and MCP (8052) SHALL be in the config

### CLI

- **REQ-CLI-001**: A Click-based CLI SHALL be available for admin/diagnostics tasks
- **REQ-CLI-002**: Commands: `contexter session create|list|get|delete`, `contexter memory create|search`, `contexter status`, `contexter export`, `contexter gc`

### Observability

- **REQ-OBS-001**: All API requests SHALL be logged with method, path, status, duration
- **REQ-OBS-002**: All bridge calls SHALL be logged with function name, args summary, duration
- **REQ-OBS-003**: All errors SHALL be logged with traceback and context

## 4. Interfaces & Data Contracts

### 4.1 Bridge Interface

```python
# contexter-server/src/core/bridge.py

class StorageEngine:
    """Async wrapper around the Rust Engine via asyncio.to_thread + ThreadPoolExecutor."""

    def __init__(self, path: str, max_workers: int = 4): ...

    # --- Session ---
    async def create_session(self, session: dict) -> dict: ...
    async def get_session(self, id: str) -> dict | None: ...
    async def list_sessions(self, filter: dict | None = None) -> list[dict]: ...
    async def update_session(self, id: str, patch: dict) -> dict | None: ...
    async def delete_session(self, id: str) -> None: ...
    async def count_sessions(self, filter: dict | None = None) -> int: ...

    # --- Memory ---
    async def create_memory(self, memory: dict) -> dict: ...
    async def get_memory(self, id: str) -> dict | None: ...
    async def search_memories(self, query: dict) -> list[dict]: ...
    async def update_memory(self, id: str, patch: dict) -> dict | None: ...
    async def delete_memory(self, id: str) -> None: ...
    async def count_memories(self, query: dict) -> int: ...

    # --- Agent ---
    async def create_agent(self, agent: dict) -> dict: ...
    async def get_agent(self, id: str) -> dict | None: ...
    async def list_agents(self, filter: dict | None = None) -> list[dict]: ...
    async def update_agent(self, id: str, patch: dict) -> dict | None: ...
    async def delete_agent(self, id: str) -> None: ...

    # --- Skill ---
    async def create_skill(self, skill: dict) -> dict: ...
    async def get_skill(self, id: str) -> dict | None: ...
    async def list_skills(self, filter: dict | None = None) -> list[dict]: ...
    async def update_skill(self, id: str, patch: dict) -> dict | None: ...
    async def delete_skill(self, id: str) -> None: ...

    # --- Settings ---
    async def get_setting(self, key: str) -> str | None: ...
    async def set_setting(self, key: str, value: str) -> None: ...

    # --- Audit ---
    async def log_audit(self, entry: dict) -> None: ...
    async def query_audit(self, filter: dict) -> list[dict]: ...

    # --- Maintenance ---
    async def flush(self) -> None: ...
    async def checkpoint(self) -> int: ...
    async def storage_size(self) -> dict: ...
    async def status(self) -> dict: ...
    async def clear_cache(self) -> None: ...
    async def cache_telemetry(self) -> dict: ...
    async def clear_cache_type(self, entity_type: str) -> None: ...
```

### 4.2 Service Interface (Example: SessionService)

```python
class SessionService:
    def __init__(self, engine: StorageEngine): ...

    async def create(self, data: SessionCreate) -> Session: ...
    async def get(self, id: str) -> Session | None: ...
    async def list(self, filter: SessionFilter | None = None) -> list[Session]: ...
    async def update(self, id: str, patch: SessionPatch) -> Session | None: ...
    async def delete(self, id: str) -> None: ...
    async def resume(self, id: str) -> Session: ...
    async def compute_efficiency(self, id: str) -> float: ...
```

### 4.3 API Contract (Example: Sessions)

```
GET  /api/v1/sessions       → 200 [{session}]
POST /api/v1/sessions       → 201 {session}
GET  /api/v1/sessions/{id}  → 200 {session} | 404
PUT  /api/v1/sessions/{id}  → 200 {session} | 404 | 422
DELETE /api/v1/sessions/{id} → 204 | 404
POST /api/v1/sessions/{id}/resume → 200 {session} | 404
```

## 5. Acceptance Criteria

See `ACCEPTANCE.md` for the full acceptance criteria catalog.

## 6. Test Automation Strategy

| Level | Location | Framework | What It Tests |
|---|---|---|---|
| Model tests | `tests/models/` | pytest + pydantic | Field validation, type coercion, serialization |
| Bridge tests | `tests/core/` | pytest + unittest.mock | Async wrapping, error propagation, large content |
| Service tests | `tests/services/` | pytest + mock bridge | Business logic, validation, error handling |
| API tests | `tests/api/` | FastAPI TestClient | HTTP endpoints, status codes, serialization |
| MCP tests | `tests/mcp/` | MCP client harness | Tool invocation, resource resolution |
| CLI tests | `tests/cli/` | Click test runner | Command execution, output format |
| Integration | `tests/integration/` | pytest + maturin build | Full stack: Rust → bridge → API |

**Coverage target:** 90%+ line coverage.

**CI integration:** `maturin develop` → `pytest --cov` as a quality gate.

## 7. Rationale & Context

The Python layer is the interface between the high-performance Rust core and the external world. It is intentionally thin — business logic lives in services, not route handlers. The bridge defers to Rust for all storage operations, so no SQL, ORM, or database drivers are needed in Python. This keeps the Python layer testable (mock the bridge) and focused on orchestration, not data management.

Port 8051/8052 were chosen because 8000–8050 is already consumed. The architecture spec originally specified 8000/8001.

## 8. Dependencies

| Dependency | Type | Version | Purpose |
|---|---|---|---|
| contexter-core | Rust crate | 0.1.0 | Rust engine compiled via maturin |
| fastapi | Python | >=0.115 | REST API framework |
| fastmcp | Python | >=0.3 | MCP server framework |
| uvicorn | Python | >=0.30 | ASGI server |
| pydantic | Python | v2 | Request/response validation |
| structlog | Python | >=0.24 | Structured logging |
| pyyaml | Python | >=6.0 | config.yaml parsing |
| click | Python | >=8.1 | CLI framework |
| httpx | Python | >=0.28 | Async HTTP (for testing) |
| pytest | Python | >=8 | Test framework |
| pytest-cov | Python | >=5 | Coverage reporting |
| pytest-asyncio | Python | >=0.24 | Async test support |

## 9. Examples & Edge Cases

See `EDGE_CASES.md`.

## 10. Validation Criteria

- All acceptance criteria in `ACCEPTANCE.md` pass
- All edge cases in `EDGE_CASES.md` are handled
- `pytest --cov` shows ≥90% line coverage
- `maturin develop` builds without error
- `cargo test --features python` passes all Rust tests
- REST API responds correctly on port 8051
- MCP server responds correctly on port 8052

## 11. Related Specifications

- [System Architecture](../../design/specs/2026-07-23-contexter-system-architecture.md)
- [Specification Hub](../../design/specs/2026-07-23-contexter-specification-hub.md)
- [Implementation Plan](../../design/plans/2026-07-23-contexter-implementation-plan.md)
