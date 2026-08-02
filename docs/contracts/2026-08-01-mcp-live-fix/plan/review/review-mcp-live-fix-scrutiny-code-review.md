# Code Review Report

# MCP Live-Functionality Repair

> Scrutiny — Code Review of Fix A (engine path) + Fix B (schema drift) on `feature/mcp-live-fix`. The MCP server previously failed **all** live calls (MagicMock stub engine + schema drift). Review covers correctness, maintainability, test quality, TDD compliance, and DDD adherence of the repaired launcher, bridge, handlers, env-var canonicalization, and resource auth templates.

**Verdict:** CONDITIONAL PASS (class: pass-with-findings)

2026-08-01 · 15 files changed (10 source/config + 5 modified tests) + 5 new test files · Code Reviewer

---

## 01 · Review Overview

| Metric | Value |
|---|---|
| Files Reviewed | 21 (10 modified src/config, 5 modified tests, 5 new tests, 1 deleted stub) |
| Tests Passed | 647 passed / 1 pre-existing failure (out of scope, documented) |
| Issues Found | 7 (2 HIGH corroborated from T6, 1 MEDIUM new, 2 LOW new, 2 informational) |
| Code Coverage | Not measured — live-protocol test coverage added for the repaired paths |

> **Scope**
> `contexter-server` MCP layer: `run_mcp.py` launcher wiring, `core/bridge.py` camelization + mock rejection, `services/memory_service.py` translation, `mcp_tools/handlers.py` type-param restore, `CONTEXTER_API_KEY` canonicalization (auth.py, mcp_server.py, api/deps.py, main.py), `{?_api_key}` resource templates, `fastmcp~=3.4.0` pin, `**/docs/tests/` gitignore, deletion of the committed `src/contexter_core.py` stub. REST API, Rust core, web UI are out of scope.

---

## 02 · Code Diff Review

All changes shown with unified diff. **16 files** changed (incl. deletion).

### contexter-server/src/contexter_core.py (DELETED)

```diff
-"""Committed Python MagicMock stub — deleted. The real Rust PyO3 extension
-(maturin wheel) now resolves for `import contexter_core`."""
```

Deletion verified on disk (`src/contexter_core.py` does not exist). This removes the root cause of the MOCK_AWAIT_ERROR class. The bridge import guard now raises a descriptive ImportError if the wheel is missing instead of silently importing a stub — correct fail-fast behavior.

### contexter-server/src/contexter_server/core/bridge.py (+87 lines)

```diff
+def _snake_to_camel(name: str) -> str:
+    head, *tail = name.split("_")
+    return head + "".join(part.capitalize() for part in tail)
+
+def _camelize_payload_keys(payload: dict) -> dict:
+    return {
+        (_snake_to_camel(key) if isinstance(key, str) else key): value
+        for key, value in payload.items()
+    }
+
 try:
     from contexter_core import Engine as _SyncEngine
-except ImportError ...
+except ImportError as exc:  # pragma: no cover - import guard
+    raise ImportError("contexter_core (the Rust PyO3 extension) is not installed. ...") from exc
```

Mock-rejection logic in `_run`:

```python
class_method = getattr(_SYNC_ENGINE_CLASS, method, None)
if class_method is None and not isinstance(_SYNC_ENGINE_CLASS, Mock):
    raise AttributeError(...)
if not isinstance(_SYNC_ENGINE_CLASS, Mock) and isinstance(class_method, Mock):
    raise TypeError("... resolves to a unittest.mock object ...")
fn = getattr(self._engine, method)
if not isinstance(self._engine, Mock) and isinstance(fn, Mock):
    raise TypeError(...)
```

Semantics are correct: wholesale mocks (explicit test doubles) are tolerated; mock *attributes on real classes/instances* (the stub-leak pattern) are rejected loudly. `_SYNC_ENGINE_CLASS` is captured at import time, before any test patching — the right pattern for validating method existence independently of a patched instance.

### contexter-server/run_mcp.py

```diff
-from contexter_core import Engine
+from contexter_server.core.bridge import StorageEngine
...
-    engine = Engine.open(engine_path)
-    services = {...}
+def build_services(engine_path: str) -> dict:
+    engine = StorageEngine(engine_path)
+    return { ... six services ... }
```

Clean extraction of `build_services` makes the launcher testable in-process (used by `test_mcp_launcher_wiring.py`). Mirrors `main._create_services` — consistent wiring across REST and MCP. DDD compliant: launcher stays a composition root, services unchanged.

### contexter-server/src/contexter_server/mcp_tools/handlers.py

```diff
-    type_filter: str | None = None,
+    type: str | None = None,
```

Restores the frozen contract parameter name (`type`) for `search_memories` (L68/L82) and `list_skills` (L159/L171), eliminating the schema/handler drift (SCHEMA_DRIFT_ERROR). FastMCP derives tool input schemas from handler signatures, so the parameter must be named `type` for the schema to advertise `type` — the revert is correct, not a regression.

### contexter-server/src/contexter_server/services/memory_service.py

```python
payload.setdefault("memory_type", "fact")
...
if "query" in query_dict:
    query_dict["keywords"] = query_dict.pop("query")
if "type" in query_dict:
    query_dict["memory_type"] = query_dict.pop("type")
```

Verified correct against models: `MemoryCreate` has no `memory_type` field (so `setdefault` is meaningful, not dead), `Memory` has `validation_alias="memoryType"` (so response parsing is symmetric with the bridge camelization), and `SearchQuery` has `type`/`query` fields matching the translation. The `setdefault` default of `"fact"` matches the domain model default — a defensible boundary translation until a role→memoryType mapping is specified.

### env-var canonicalization (auth.py, mcp_server.py, api/deps.py, main.py, test_security.py, test_mcp_auth.py, test_mcp_server.py)

All src references to the misspelled `CONtexTER_API_KEY` are gone (grep confirmed zero src matches). Tests updated consistently; new `TestEnvVarCanonicalName` regression class asserts both layers read the canonical name and that the legacy misspelling no longer gates anything.

### contexter-server/src/contexter_server/mcp_server.py

```diff
-    @mcp.resource("contexter://session/{id}")
+    @mcp.resource("contexter://session/{id}{?_api_key}")
```

RFC 6570 query block added to session/memory/agent templates; analytics already had it. Live-protocol tests prove correct key succeeds and missing/wrong key is rejected through the in-process FastMCP client.

### contexter-server/pyproject.toml · .gitignore

`fastmcp>=0.3` → `fastmcp~=3.4.0` (verified working version, `<3.5.0` guard). Root `.gitignore` gains `**/docs/tests/` — matches the ephemeral scratch-dir convention.

Diff data: `git diff` working tree vs HEAD (feature branch, no commits)

---

## 03 · Review Findings

### HIGH-1 (corroborates T6 F1) — Agent response schema drift NOT addressed

`services/agent_service.py` still does `Agent.model_validate(raw)` on the engine response. T6 live verification showed the engine returns `{id, name, type, description, capabilities, status, config, version, createdAt, updatedAt}` while the pydantic `Agent` model requires `provider`/`model` → `ValidationError` on `get_agent_info` and `contexter://agent/{id}`. **Nothing in this changeset touches the agent response path** — the camelization layer only fixes request payloads. `AgentService.create` is likewise broken (engine rejects payloads missing `type`/`description`). Correctly documented for the Auto Bug Loop, but it remains a real HIGH in the live call path.

**Suggestion:** separate bug contract: engine→domain response mapper for Agent (by_alias extraction or a translation layer), plus create-payload mapping. Add a live round-trip test (`test_engine_real`-style) for agent get/create.

### HIGH-2 (corroborates T6 F2) — Skill response schema drift NOT addressed

Same pattern: `skill_service.py` does `Skill.model_validate(raw)` / `SkillCreate` requires `type: str, version: Optional[str]` while the engine returns `category` and integer `version`. `list_skills` fails live. No code in this changeset addresses it. Documented for the Auto Bug Loop; corroborated by inspection.

### MEDIUM-1 (new) — Uniform camelization is only live-verified for 4 of 34 engine methods

`_camelize_payload_keys` is applied indiscriminately to **every** bridge method: create/update/delete/count/list for sessions, memories, agents, skills, plus `log_audit`, `query_audit`. Live round-trip tests (`test_engine_real.py`, launcher wiring) prove the camelCase contract only for `create_session`, `create_memory`, `search_memories`, `status`. All other methods are asserted **only against mocks that assert the camelized form** — a circular verification: if the engine's `log_audit`/`query_audit`/`count_*` serde structs keep snake_case fields (no `rename_all`), those calls would fail live while the suite stays green.

**Why it matters:** this is exactly the class of bug that shipped the original defect (mock-verified unit tests passing while live calls failed).

**Suggestion:** add live round-trips (temp-dir engine) for at least one method per family — `update_memory`/`update_agent`, `count_memories`, `log_audit`/`query_audit`, `list_sessions` with filters — or scope camelization per-method to the proven contract.

### LOW-1 (new) — `CONtexTER_BRIDGE_POOL_SIZE` retains the misspelled naming style this fix canonicalized away

`bridge.py::__init__` reads `os.environ.get("CONtexTER_BRIDGE_POOL_SIZE", "")` — the same mixed-case legacy style that was just canonicalized from `CONtexTER_API_KEY` → `CONTEXTER_API_KEY` across four files. Pre-existing, but inconsistent with the fix's stated principle.

**Suggestion:** read `CONTEXTER_BRIDGE_POOL_SIZE` with a fallback to the legacy name (or document the difference explicitly).

### LOW-2 (new) — Leftover scratch files under `docs/tests/` and `contexter-server/docs/tests/`

12+ files remain on disk (probe scripts, raw protocol logs, JSON results, markdown probe reports). The gitignore addition works (verified via `git check-ignore` — they will not be committed), but the implementation summary claims "scratch files in docs/tests/ created and deleted by Workers" — contradicted by the working tree. Per the workflow, this directory is ephemeral and MUST be cleaned up.

**Suggestion:** delete `docs/tests/` and `contexter-server/docs/tests/` before shipping.

### LOW-3 (new) — Broad exception matching in live resource-auth tests

`pytest.raises(Exception, match="API key required")` in `test_mcp_resource_auth_live.py` will pass if any unrelated exception happens to contain that message. Pragmatic for FastMCP's error wrapping, but a narrower assertion (e.g., match the MCP protocol error shape) would make the tests more precise.

### INFO-1 — Hard runtime/test dependency on the Rust wheel (intentional, documented)

The bridge import guard converts a missing wheel into an explicit, actionable ImportError, and `test_engine_real.py` hard-imports `contexter_core` (collection error if absent). This is the intended fail-fast stance, but it means CI **must** install the wheel or the entire suite fails to collect. Worth an explicit CI note.

### INFO-2 — `type` parameter shadows the builtin in handlers (contract-mandated)

The original `type_filter` rename avoided shadowing `type()`, but FastMCP derives tool schemas from handler signatures, so the frozen contract (`type`) requires the shadow. The REST API avoids this with `type_filter` + `alias="type"` (FastMCP has no equivalent alias mechanism here). Acceptable; flagging for awareness only — do NOT rename.

### NIT-1 — Private-attribute access in launcher wiring tests

`test_mcp_launcher_wiring.py` reaches into `service._engine` and `engine._engine`, and uses `__import__("unittest").mock.Mock` instead of a module-top import. Works and is purposeful (proving no mock leaks into the live path), but couples tests to internals.

---

## 04 · Summary & Recommendations

> **Code Quality Assessment**
> Strong repair. The root-cause fix is architecturally sound: the stub is deleted, the launcher is rewired through the same `StorageEngine` bridge the REST API uses (composition-root consistency, DDD-respecting), the bridge gained a genuinely defensive mock-rejection layer with correct wholesale-mock tolerance, and the schema-drift fix restores the frozen contract. The biggest improvement is test strategy: new tests exercise the real FastMCP protocol path via in-process `fastmcp.Client` — these would have caught the original bug, unlike the handler-only tests. TDD evidence is credible (documented RED 12-fail → GREEN, 14/14 new tests, 647/1 suite). The two T6 HIGH findings (Agent/Skill response drift) are corroborated and correctly deferred to the Auto Bug Loop as separate contracts. The main new concern is MEDIUM-1: the camelization contract is assumed, not proven, for ~30 of 34 engine methods — the same verification gap class that produced the original bug.

> **Strengths**
> - Root-cause elimination: stub deleted (verified), real wheel resolves, bridge refuses mock attributes at class AND instance level with a clear error.
> - `_SYNC_ENGINE_CLASS` captured before patching — correct pattern for class-level validation.
> - Live-protocol tests (in-process FastMCP client) for tools, resources, and auth — a qualitative jump in regression protection.
> - Env-var canonicalization complete across src; regression tests assert the legacy name no longer gates auth.
> - `{?_api_key}` templates tested positively and negatively on all three repaired resources (analytics regression-guarded).
> - `build_services` extraction makes the launcher unit-testable; wiring test proves all six services hold the bridge, never a raw/mock engine.
> - Bridge camelization is documented with a clear boundary contract (top-level only; nested `metadata` pass through).
> - `fastmcp~=3.4.0` pin aligns dependencies with the verified working version; `**/docs/tests/` gitignore matches the ephemeral-scratch convention.

> **Recommended Improvements**
> 1. (HIGH, Auto Bug Loop) Fix Agent and Skill response/create mapping against the real engine contract; add live round-trips.
> 2. (MEDIUM) Prove the camelization contract per method family with temp-dir engine round-trips (update/count/audit paths) or scope camelization per method.
> 3. (LOW) Canonicalize `CONtexTER_BRIDGE_POOL_SIZE` or document the exception.
> 4. (LOW) Delete leftover `docs/tests/` scratch files (both locations) before shipping.
> 5. (LOW) Narrow the `pytest.raises(Exception, ...)` matching in resource-auth live tests.
> 6. (NIT) Consider a public mock-probe on the bridge to avoid private-attribute access in tests.

---

_Generated by Code Reviewer · 2026-08-01 · Validation Contract: 2026-08-01-mcp-live-fix_
