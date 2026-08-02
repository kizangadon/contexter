# Code Review Report

# MCP Live Fix — Iteration 3 Code Review (count-sessions fast path + FastMCP framework stderr policy)

> Auto Bug Loop iteration 3 over two bug contracts: (1) 2026-08-01-count-sessions-fast-path — O(1) estimate-num-keys fast path for unfiltered count_sessions (mirroring count_agents/count_skills), filtered counts stay exact via index-prefix scan; (2) 2026-08-01-fastmcp-framework-logging — logging.Filter on the fastmcp namespace suppressing the 2672-char rich traceback box on failure stderr (REQ-FL-001..005), installed at package import.

**Verdict:** CONDITIONAL PASS (class: PASS-WITH-FINDINGS — 1 LOW (latent), 2 NIT, 3 INFO; no blockers; all changes verified by reviewer-run tests (32 Python + 25 Rust). Findings feed the Auto Bug Loop.)

2026-08-02 · 7 files changed · Code Reviewer

---

## 01 · Review Overview

| Metric | Value |
|---|---|
| Files Reviewed | 7 (iter-3 scope: fastmcp_logging.py, contexter_server/__init__.py, test_framework_efs_stderr.py, test_bridge_live_coverage.py, rocksdb.rs, session_test.rs, agent_skill_test.rs) |
| Tests Passed | 32 Python (13 framework-EFS + 19 live coverage) + 25 Rust (9 session + 16 agent_skill) — reviewer-run; full-suite baseline at iter-2: 867 Python + 336 Rust |
| Issues Found | 6 |
| Code Coverage | N/A (targeted review run)% |

> **Scope**
> Scrutiny of the iter-3 change set only (the two bug contracts above), with regression checks against the iter-1/iter-2 baseline reports. Static analysis of the installed fastmcp 3.4.0 emitter inventory and pytest 8.4.2 logging plugin, plus direct test execution of the affected suites. Zero implementation files touched; read-only verification.

---

## 02 · Code Diff Review

All changes shown with unified diff. **7 files** changed.

### contexter-server/src/contexter_server/fastmcp_logging.py (new)

```diff
+"""FastMCP framework logging policy: bounded failure stderr (REQ-FL-001)."""
+
+import logging
+
+_FRAMEWORK_ERROR_PREFIXES = (
+    "Error calling tool ",
+    "Error reading resource ",
+    "Error rendering prompt ",
+)
+
+_INSTALLED_ATTR = "_contexter_bounded_stderr_filter_installed"
+
+_EMITTER_LOGGERS = (
+    "fastmcp",
+    "fastmcp.server",
+    "fastmcp.server.server",
+)
+
+
+class _SuppressFrameworkTracebackBox(logging.Filter):
+    """Suppress framework error-call records (no stderr output for failures)."""
+
+    def filter(self, record: logging.LogRecord) -> bool:
+        if record.getMessage().startswith(_FRAMEWORK_ERROR_PREFIXES):
+            return False
+        return True
+
+
+def configure_fastmcp_failure_stderr() -> None:
+    """Install the bounded-stderr filter on the FastMCP emitter loggers."""
+    for name in _EMITTER_LOGGERS:
+        logger = logging.getLogger(name)
+        if getattr(logger, _INSTALLED_ATTR, False):
+            continue
+        logger.addFilter(_SuppressFrameworkTracebackBox())
+        setattr(logger, _INSTALLED_ATTR, True)
+
+# contexter_server/__init__.py:
+#   from contexter_server.fastmcp_logging import configure_fastmcp_failure_stderr
+#   configure_fastmcp_failure_stderr()
```

Diff data: `[{"file":"contexter-server/src/contexter_server/fastmcp_logging.py","content":"NEW FILE: filter + configure_fastmcp_failure_stderr(), wired at package import (stdlib-only, idempotent via _INSTALLED_ATTR)"},{"file":"contexter-core/src/storage/rocksdb.rs","content":"+        // When no filters are set, use the RocksDB estimate-num-keys property\n+        // for a fast O(1) count instead of a full scan (mirrors count_agents\n+        // and count_skills). The sessions CF holds only session keys, so the\n+        // estimate counts exactly the session rows.\n+        if filter.agent_id.is_none() && filter.status.is_none() {\n+            if let Some(val) = self.db.property_value_cf(\n+                self.cf(self.cfs.sessions)?, \"rocksdb.estimate-num-keys\"\n+            ).ok().flatten() {\n+                if let Ok(count) = val.parse::<u64>() { return Ok(count); }\n+            }\n+            // Fall through to full scan if the property is unavailable.\n+        }"},{"file":"contexter-core/tests/engine/session_test.rs","content":"-    let total = engine.count_sessions(&SessionFilter::default())...\n-    assert_eq!(total, 100, ...);\n+    // NOTE: the unfiltered count_sessions is now an O(1) estimate\n+    // (rocksdb.estimate-num-keys, REQ-CS-001) that can lag updates...\n+    let all = engine.list_sessions(&SessionFilter::default())...;\n+    assert_eq!(all.len(), 100, \"should have 100 sessions across all threads\");"},{"file":"contexter-core/tests/engine/agent_skill_test.rs","content":"+6 new tests: count_agents (unfiltered + status), count_skills (unfiltered + category), count_sessions (store parity, project filter)"},{"file":"contexter-server/tests/mcp/test_framework_efs_stderr.py","content":"NEW FILE: 13 tests — 9 live error classes through real FastMCP path with capfd, BASELINE_FRAMES pinned; concurrency test (n=5); success path; diagnostics log retains traceback; 2 filter unit tests"},{"file":"contexter-server/tests/core/test_bridge_live_coverage.py","content":"+    async def test_count_sessions_matches_seeded_store_and_overview(self, engine):\n+        \"\"\"REQ-CS-004 / AC-CS-005: unfiltered count and get_overview report the\n+        seeded session count exactly (12-session store -> 12).\n+        \"\"\"\n+        for i in range(12):\n+            await engine.create_session(_new_session(project=f\"ov-{i % 3}\"))\n+        assert await engine.count_sessions() == 12\n+        assert await engine.count_sessions({}) == 12\n+        assert await engine.count_sessions({\"project\": \"ov-0\"}) == 4\n+        overview = await AnalyticsService(engine).get_overview()\n+        assert overview.total_sessions == 12"}]`

---

## 03 · Review Findings

## Findings (Auto Bug Loop iter-3)

**Verdict note:** 6 findings total — no blockers, no highs. All changes verified by reviewer-run tests (32 Python + 25 Rust targeted).

### F-1 🔴→🟡 LOW (latent gap): prompt emitter logger not covered by the filter
**File:** `contexter-server/src/contexter_server/fastmcp_logging.py:47-51` (`_EMITTER_LOGGERS`)

**Evidence:** Complete emitter inventory of the installed fastmcp 3.4.0 (`rg 'Error (calling tool|reading resource|rendering prompt)'`): every `server.py` emitter — `1297` (tool), `1428/1431/1472/1475` (resource), `1591/1594` (prompt), plus the `e.log_level` paths `1285/1468/1587` — uses `get_logger(__name__)` → logger `fastmcp.server.server`, which IS covered. The ONLY uncovered emitter is `fastmcp/prompts/function_prompt.py:370`: `logger.exception(f"Error rendering prompt {self.name}")` on logger `fastmcp.prompts.function_prompt`.

**Why it matters:** Python's logging applies only the ORIGINATING logger's filters (verified empirically: a record emitted on `fastmcp.prompts.function_prompt` still reaches the root handler with a filter on `fastmcp`). The `"Error rendering prompt "` prefix in `_FRAMEWORK_ERROR_PREFIXES` is therefore dead code for its actual emitter. Latent today: contexter registers zero prompts (no `add_prompt`/`@prompt` anywhere under `contexter_server/src` — verified), so REQ-FL-001 is not violated live. The day a prompt is registered, a prompt-render failure would silently render the 2672-char rich box on stderr.

**Suggestion:** add `"fastmcp.prompts.function_prompt"` (or the `fastmcp.prompts` namespace) to `_EMITTER_LOGGERS`; optionally add a drift test that inventories the installed fastmcp for emitters of the three prefixes and asserts each originating logger is covered.

### F-2 🟡 NIT: exactness at the default-limit boundary
**File:** `contexter-core/tests/engine/session_test.rs` (`test_concurrent_operations`)

`list_sessions(&SessionFilter::default())` carries `limit: 100` (`models/session.rs` default), and the test asserts exactly 100. The "exact full scan" claim holds only while the seeded count equals the default limit; a future extension beyond 100 sessions would silently truncate and the assertion would no longer prove "no lost writes". Suggestion: `SessionFilter { limit: u64::MAX, ..Default::default() }` to keep exactness independent of the default.

### F-3 🟡 NIT: redundant assertion
**File:** `contexter-server/tests/mcp/test_framework_efs_stderr.py:297`

`assert len(stderr) <= n * _STDERR_LIMIT` is strictly weaker than the immediately preceding `_assert_bounded(stderr)` (total ≤ 512) for any n ≥ 1. Harmless; consider removing or repurposing (e.g., a per-failure budget derived from captured records).

### F-4 ⚪ INFO: capfd observes framework-only output in-process — docstring precision
**File:** `contexter-server/tests/mcp/test_framework_efs_stderr.py:19-23`

Docstring claims capfd observes the "TOTAL emitted for a failure ... exactly as AC-FL-001 defines it ('bridge line + any framework output')". Static analysis of pytest 8.4.2 (`_pytest/logging.py`: `LoggingPlugin.pytest_runtest_call` wraps EVERY test in `_runtest_for`, which attaches a root `LogCaptureHandler` at NOTSET via `catching_logs`) plus the passing 32-test run show the in-process measurement is framework-only: the bridge's structlog ERROR records (LoggerFactory → stdlib logger `contexter_server.core.bridge`, propagate=True) are captured by pytest's root handler and never reach fd-2; `logging.lastResort` never fires. The bridge-line budget is exercised by the live subprocess/user-testing paths instead. Not a defect — the tests remain fully discriminating (without the filter the rich box reaches fd-2 via the fastmcp namespace handler, `propagate=False`, which pytest cannot capture, and fails `_assert_bounded`). Suggestion: clarify the docstring to state the in-process capfd measurement covers the framework contribution.

### F-5 ⚪ INFO: estimate fast path has no prefix guard — documented invariant
**File:** `contexter-core/src/storage/rocksdb.rs` (count_sessions fast path; same pattern in count_agents/count_skills)

The estimate counts every key in the CF; correctness relies on the invariant "the sessions CF holds only session keys" — verified true today (all writes via `session_key`; delete removes both CFs) and documented in the test docstring (`test_bridge_live_coverage.py:37-45`). A future writer of any other key type into that CF would silently inflate unfiltered counts. Consider a code comment asserting the invariant next to the fast path.

### F-6 ⚪ INFO: prefix filter drops records at every level (incl. fastmcp's own suppressed-frame paths)
**File:** `contexter-server/src/contexter_server/fastmcp_logging.py:70-73`

Any record on covered loggers whose message starts with the three prefixes is dropped regardless of level — including fastmcp's `e.log_level` + `exc_info` paths (`server.py:1285/1468/1587`). Consistent with the documented "framework contributes zero bytes" goal (even a single wrapped RichHandler line can exceed 512 for long URIs) and with the FastMCPError semantic; the unit test pins WARNING/INFO pass-through for non-matching messages. No change needed — recorded for completeness. Positive controls verified: idempotency via `_INSTALLED_ATTR`; package-import wiring is stdlib-only (no fastmcp import → no import-order coupling); filters survive fastmcp `configure_logging` (removes handlers only).

### Baseline regression check (iter-1 / iter-2)
All prior findings remain resolved: no `docs/tests/` scratch directories exist (root or contexter-server); iter-2-reviewed paths (`count_sessions` bridge wrapper, bridge stderr diagnostics) are unchanged by iter-3; the session concurrency invariant is now asserted exactly via `list_sessions` and passes (9/9).

---

## 04 · Summary & Recommendations

> **Code Quality Assessment**
> Focused, well-documented change. The Rust fast path is a faithful mirror of the established count_agents/count_skills pattern with a safe fall-through to the full scan, and the Python filter policy is minimal, idempotent, and correctly targeted at the fastmcp namespace (the only reachable layer, given propagate=False). The test suites are genuinely discriminating: every error class is exercised through the real FastMCP call path, client-visible isError frames are pinned byte-identical, and the concurrency test was verified sound by direct execution (32/32 Python). The two real weaknesses are a latent prompt-emitter coverage gap and a handful of precision/robustness nits — none of which affect current behavior. Overall: strong, shippable-after-loop iteration.

> **Strengths**
> - test_concurrent_failures_each_bounded is sound: reviewer-executed (5 concurrent invalid-id failures) with stderr ≈ 0 in-process — bridge lines are captured by pytest's root handler, framework records are dropped by the filter; without the filter the 2672-char box fails it.
- Emitter inventory matches _FRAMEWORK_ERROR_PREFIXES exactly for all server.py paths in fastmcp 3.4.0.
- 12-session overview test (REQ-CS-004/AC-CS-005) respects EC-CS-003 semantics (fresh store → estimate exact) and passes.
- _assert_bounded checks char count AND byte count, box chars, raw traceback, and source frames.
- Package-import wiring is safe: stdlib-only, idempotent, covers every entry point; no fastmcp import at module top.
- session_test.rs comment honestly documents why the concurrency assertion moved from count_sessions to list_sessions.
- Docstring policy in test_bridge_live_coverage.py (lines 37-45) is an accurate, documented contract for estimate semantics; the changed delete assertion complies with it (passes).

> **Recommended Improvements**
> - Add 'fastmcp.prompts.function_prompt' to _EMITTER_LOGGERS (or add a drift test asserting emitter-logger coverage) — F-1.
- Use an explicit large limit in test_concurrent_operations so exactness does not depend on the default limit — F-2.
- Remove the redundant n * _STDERR_LIMIT assertion — F-3.
- Clarify the EFS docstring: in-process capfd measures the framework contribution; bridge-line budget is covered by the subprocess suite — F-4.
- Add a code comment asserting the sessions-CF-holds-only-session-keys invariant next to the estimate fast path — F-5.

---

_Generated by Code Reviewer · 2026-08-02 · Validation Contract: 2026-08-01-mcp-live-fix (iter-3)_
