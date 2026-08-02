# Code Review Report

# MCP Live-Fix — Auto Bug Loop Iteration 1 Code Review

> Iteration-1 re-validation of the MCP live-fix change set (feature/mcp-live-fix @ 27e031d + uncommitted working tree): bridge camelization, agent/skill schema-drift repair, handler error contract, validation bounds, launcher hardening, env-var canonicalization, scratch cleanup, and 18 bug contracts.

**Verdict:** CONDITIONAL PASS (class: 3 low-severity findings; all 7 baseline findings resolved)

2026-08-01 · 60 files changed · Code Reviewer

---

## 01 · Review Overview

| Metric | Value |
|---|---|
| Files Reviewed | 31 |
| Tests Passed | 794 |
| Issues Found | 3 |
| Code Coverage | n/a% |

> **Scope**
> Full re-validation of the parent MCP live-fix contract (SPEC.md REQ-007/AC-1..AC-7, ACCEPTANCE.md, EDGE_CASES.md) plus all 18 bug contracts. Source verified: mcp_tools/handlers.py, mcp_tools/errors.py, mcp_tools/auth.py, mcp_server.py, core/bridge.py, models/agent.py, models/skill.py, services/{agent,skill,memory,session,analytics}_service.py, rate_limiter.py, main.py, api/deps.py, cli/status_commands.py, run_mcp.py, and the Rust ground truth (contexter-core models/agent.rs, models/skill.rs, storage/rocksdb.rs, bridge.rs). Tests executed: full contexter-server suite (794 passed, 0 failed).

---

## 02 · Code Diff Review

All changes shown with unified diff. **60 files** changed.

### Representative diff — mcp_tools/handlers.py (error contract + validation)

```diff
--- a/contexter-server/src/contexter_server/mcp_tools/handlers.py
+++ b/contexter-server/src/contexter_server/mcp_tools/handlers.py
@@ handler error contract @@
+def _raise_structured_error(log, error: HandlerError, started) -> NoReturn:
+    duration_ms = round((time.monotonic() - started) * 1000, 3)
+    log.error("handler_error", error_kind=error.kind, duration_ms=duration_ms)
+    raise error
+def _validate_content(content, log, started) -> None:
+    if not content or not content.strip():
+        _raise_structured_error(log, validation_error("content must not be empty"), started)
+    if len(content) > MAX_CONTENT_LENGTH:
+        _raise_structured_error(log, validation_error(
+            f"content exceeds maximum length of {MAX_CONTENT_LENGTH}"), started)
+def _clamp_session_list_limit(value):
+    if value is None:
+        return None
+    return max(0, min(value, MAX_SESSION_LIST_LIMIT))
```

Diff data: `60 files changed: 1 D, 1 A (src), 18 A (tests), 40 M. Full inventory in git status (feature/mcp-live-fix working tree).`

---

## 03 · Review Findings

**Baseline (Phase 4) findings — all 7 resolved** (HIGH-1/HIGH-2 agent+skill schema drift, MEDIUM-1 camelization coverage, LOW-1 env-var canonicalization, LOW-2 scratch files, LOW-3 broad exception matching, INFO-1/2, NIT-1): verified resolved against source and live-engine tests.

| # | Severity | Location | Description | Contract |
|---|----------|----------|-------------|----------|
| 1 | P3 (low) | models/agent.py L43-46, models/skill.py L43-58 | pydantic 2.13.4 emits `UnsupportedFieldAttributeWarning` (5 occurrences in suite) for `validation_alias=AliasChoices(...)` inside `Field()` when FastAPI wraps the models. Functionally verified WORKING (legacy `tools` -> `capabilities` maps via real FastAPI path, 201; engine `category`/`filePath` parse correctly) — the warning signals a fragile pattern, not a functional failure. | agent-skill-schema-drift |
| 2 | P3 (low) | mcp_tools/handlers.py L168,254,317,442,470,497 | `not_found_error(id)` interpolates the raw caller-supplied `id` into the error message in handlers that do not UUID-validate first (get_session, get_agent_info, session/memory/agent resources). REQ-IV-005 requires error messages not echo unbounded client input; `_bounded()` exists and is used for validation messages but not not-found messages. Frozen `Resource not found: <id>` convention is preserved for normal ids — apply `_bounded(id)` (64-char cap) while keeping the convention. | input-validation-gaps (REQ-IV-005), error-shape-drift |
| 3 | P3 (low) | contexter-server/docs/tests/ | REQ-SC-001 not satisfied at review time: 8 scratch files remain (e2e_iter1.py, e2e_iter1_err.txt, e2e_iter1_out.txt, probes_iter1.py, probes_iter1_err.txt, probes_iter1_out.txt, probes_iter1.json, results_iter1.json). Headers declare 'Scratch file — deleted after validation.' Files appear to belong to the concurrently-running User-Testing Validator's live E2E harness (created minutes before this review); not deleted here to avoid sabotaging the parallel run, but MUST be removed before the iteration closes. | scratch-cleanup (REQ-SC-001) |

---

## 04 · Summary & Recommendations

> **Code Quality Assessment**
> High quality overall. The change set is disciplined: every contract has TDD reproduction tests (18 new test files, 2868 lines), live-engine coverage (test_bridge_live_coverage.py exercises 35/36 contract methods with locked wire shapes), and the error contract is uniformly implemented (HandlerError raising, no success-frame smuggling, kind-only structured logging). Translation boundaries (agent config blob, skill category, memory keywords, analytics telemetry) are documented in module docstrings and verified against Rust serde ground truth. 794 tests pass (baseline 647 + 1 pre-existing failure now fixed). Three low-severity items remain: pydantic alias-warning noise, unbounded id echo in not-found messages, and incomplete scratch cleanup (REQ-SC-001).

> **Strengths**
> - Frozen error contract uniformly enforced: every failure path raises HandlerError; never a `{"error": ...}` success payload; kind-only structured logging with correlation ids (REQ-HO-001/002).
- Bridge mock-rejection (TypeError on unittest.mock methods) and ImportError guard make stub shadowing impossible (test_engine_real.py, test_bridge_mock_rejection.py).
- Byte-identity large-content path (>=102400) tested live against the real Rust engine, including the exact threshold boundary and single-encode guarantee (REQ-BD-002).
- Skill type filter verified end-to-end: service translates type->category AND re-filters as defense in depth; engine applies category case-insensitively (rocksdb.rs L1262-1265).
- Live coverage harness locks the engine wire shapes (Session no name, Agent key type, Skill key category, MemoryQuery keywords) preventing silent serde drift.
- Env-var canonicalization completed repo-wide (CONTEXTER_API_KEY in deps.py, auth.py, mcp_server.py) with a scanning test asserting zero `CONtexTER_` typos.
- Launcher failure behavior documented, tested (exit 2, single clean stderr line, server-side diagnostics log).

> **Recommended Improvements**
> 1. Move `validation_alias` out of `Field()` into `Annotated` metadata (or model-level alias handling) to silence pydantic 2.13 warnings and future-proof the alias pattern.
2. Bound the id in `not_found_error()` call sites with `_bounded(id)` so REQ-IV-005 holds for all error messages while preserving the frozen `Resource not found: <id>` convention.
3. Complete REQ-SC-001: delete the 8 remaining scratch files under contexter-server/docs/tests/ once the parallel User-Testing run finishes.

---

_Generated by Code Reviewer · 2026-08-01 · Validation Contract: 2026-08-01-mcp-live-fix_
