---
title: "Phase 3 — Python API Layer: Acceptance Criteria"
version: 1.0
date_created: 2026-07-25
tags: acceptance-criteria, given-when-then
---

# Acceptance Criteria — Phase 3: Python API Layer

## AC-001: Python project skeleton exists

**Given** the project root
**When** examining `contexter-server/`
**Then** it SHALL contain `pyproject.toml`, `src/main.py`, `src/mcp_server.py`, `src/__init__.py`, and `tests/` directory

---

## AC-002: Maturin build config exists

**Given** the project root
**When** examining `contexter-core/`
**Then** it SHALL contain a `pyproject.toml` configured for maturin build
**And** running `maturin develop --release -m contexter-core/pyproject.toml` SHALL succeed
**And** `python -c "import contexter_core"` SHALL succeed

---

## AC-003: Module tree mirrors domain bounded contexts

**Given** the `contexter-server/src/` directory
**Then** it SHALL contain subdirectories: `api/`, `services/`, `models/`, `core/`, `mcp_tools/`, `cli/`
**And** each SHALL have an `__init__.py`

---

## AC-004: Pydantic models exist for all entities (DDD)

**Given** `contexter-server/src/models/`
**Then** it SHALL contain model files for: `session`, `memory`, `agent`, `skill`, `analytics`, `settings`, `audit`, `search`, `export`, `correlation`, `notifications`
**And** each model SHALL use Pydantic v2 `BaseModel` with type-annotated fields
**And** each model SHALL use ubiquitous language consistent with the domain

---

## AC-005: Model validation tests pass

**Given** the model test files
**When** running `pytest tests/models/`
**Then** all tests SHALL pass
**And** models SHALL reject invalid field types with `ValidationError`
**And** models SHALL coerce compatible types correctly
**And** serialization round-trips (model → dict → model) SHALL preserve all fields

---

## AC-006: Core bridge exists with correct import

**Given** `contexter-server/src/core/bridge.py`
**Then** it SHALL import: `from contexter_core import Engine`
**And** it SHALL define a `StorageEngine` class wrapping the Rust `Engine`
**And** all Rust calls SHALL go through `asyncio.to_thread()` with a `ThreadPoolExecutor`

---

## AC-007: Bridge CRUD operations work

**Given** the StorageEngine
**When** calling create → get → update → delete on sessions, memories, agents, skills
**Then** each operation SHALL return the correct data type per REQ-BRG-006
**And** errors from Rust SHALL be propagated as Python exceptions

---

## AC-008: Bridge supports large content paths

**Given** the StorageEngine
**When** storing memory content > 100KB
**Then** the bridge SHALL use a direct PyBytes path (not double JSON encoding)
**And** the content SHALL be retrievable with byte fidelity

---

## AC-009: Bridge tests pass (TDD)

**Given** `tests/core/`
**Then** it SHALL contain storage engine tests
**When** running `pytest tests/core/`
**Then** all tests SHALL pass
**And** tests SHALL cover: all CRUD operations, error propagation, large content path, thread pool behavior

---

## AC-010: Service layer exists for all bounded contexts

**Given** `contexter-server/src/services/`
**Then** it SHALL contain service files for: `session_service`, `memory_service`, `agent_service`, `skill_service`, `analytics_service`, `search_service`, `export_service`, `notification_service`, `audit_service`, `correlation_service`, `onboarding_service`, `settings_service`
**And** each service SHALL accept a `StorageEngine` via constructor injection

---

## AC-011: Service tests pass (TDD, mocked bridge)

**Given** `tests/services/`
**When** running `pytest tests/services/`
**Then** all tests SHALL pass
**And** tests SHALL use a mocked `StorageEngine` (no actual Rust calls)
**And** tests SHALL verify business logic independently of the bridge

---

## AC-012: FastAPI server starts on port 8051

**Given** `uvicorn contexter-server.src.main:app --port 8051`
**When** starting the server
**Then** it SHALL bind to port 8051
**And** `GET /health` SHALL return `200 {"status": "ok"}`

---

## AC-013: All REST endpoints are under /api/v1/

**Given** the FastAPI application
**When** examining the OpenAPI schema at `GET /openapi.json`
**Then** all Contexter endpoints SHALL be under `/api/v1/`

---

## AC-014: REST endpoint groups exist per spec

**Given** the FastAPI application
**When** examining route definitions
**Then** the following endpoint groups SHALL exist:
- `/api/v1/sessions` — CRUD + resume
- `/api/v1/memories` — CRUD + search + versions
- `/api/v1/agents` — CRUD
- `/api/v1/skills` — CRUD
- `/api/v1/analytics/` — overview, health, performance, resources, costs, services
- `/api/v1/efficiency/` — overview, memory, sessions, agents, skills, tokens, correlation
- `/api/v1/search` — search
- `/api/v1/settings` — get/put sections
- `/api/v1/notifications` — list, mark read, read all
- `/api/v1/audit` — query
- `/api/v1/files` — read, diff, watch
- `/api/v1/correlation` — overview, timeline, compare
- `/api/v1/export` — submit, status, download, history
- `/api/v1/feedback` — bug, suggest
- `/api/v1/onboarding` — status, wizard, progress
- `/api/v1/changelog` — get

---

## AC-015: Route handlers delegate to service layer

**Given** any route handler
**Then** it SHALL NOT contain business logic inline
**And** it SHALL delegate to the corresponding service method

---

## AC-016: API tests pass (TDD, TestClient)

**Given** `tests/api/`
**When** running `pytest tests/api/`
**Then** all tests SHALL pass
**And** tests SHALL use FastAPI `TestClient` with dependency overrides

---

## AC-017: MCP server starts on port 8052

**Given** the MCP application
**When** starting on port 8052 with SSE transport
**Then** it SHALL bind to port 8052
**And** SHALL respond to SSE connections

---

## AC-018: MCP tools exist per spec

**Given** the MCP server
**Then** tools SHALL include: `store_memory`, `search_memories`, `get_session`, `list_recent_sessions`, `get_agent_info`, `list_skills`, `get_system_health`, `export_data`

---

## AC-019: MCP resources exist per spec

**Given** the MCP server
**Then** resources SHALL include: `contexter://session/{id}`, `contexter://memory/{id}`, `contexter://agent/{id}`, `contexter://analytics/overview`

---

## AC-020: MCP tests pass

**Given** `tests/mcp/`
**When** running `pytest tests/mcp/`
**Then** all tests SHALL pass

---

## AC-021: Settings service reads/writes config.yaml

**Given** `settings_service.py`
**When** reading settings
**Then** it SHALL read from `~/.contexter/config.yaml`
**And** SHALL create the file with defaults if it does not exist
**And** SHALL write settings back correctly

---

## AC-022: CLI exists with core commands

**Given** the CLI entry point
**Then** commands SHALL include: `contexter session (create|list|get|delete)`, `contexter memory (create|search)`, `contexter status`, `contexter export`, `contexter gc`
**And** `contexter --help` SHALL display all commands

---

## AC-023: CLI diagnostics tests pass

**Given** `tests/cli/`
**When** running `pytest tests/cli/`
**Then** all tests SHALL pass

---

## AC-024: Observability logging exists

**Given** the running servers
**Then** ALL API requests SHALL be logged with method, path, status, and duration
**And** ALL bridge calls SHALL be logged with function name, args summary, and duration
**And** ALL errors SHALL be logged with traceback and context

---

## AC-025: Full test suite passes with ≥90% coverage

**Given** the project
**When** running `pytest --cov=contexter-server.src --cov-fail-under=90 tests/`
**Then** the suite SHALL pass
**And** line coverage SHALL be ≥90%

---

## AC-026: DDD ubiquitous language enforced (no anti-patterns)

**Given** all Python source files in `contexter-server/src/`
**Then** module names, class names, method names, and parameter names SHALL reflect domain concepts
**And** NO generic terms like "manager", "util", "helper", "common" SHALL appear in module or class names
