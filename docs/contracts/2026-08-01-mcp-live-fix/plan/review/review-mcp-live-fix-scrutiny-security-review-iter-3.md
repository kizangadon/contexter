# Security Review Report

# MCP Server Live-Functionality Repair — Auto Bug Loop Iteration 3

> Security re-review of the ENTIRE feature scope after iter-3 bug-contract fixes: (1) `count_sessions` unfiltered O(1) fast path via `rocksdb.estimate-num-keys` (contract `2026-08-01-count-sessions-fast-path`), (2) FastMCP framework failure-stderr suppression via a new `logging.Filter` (contract `2026-08-01-fastmcp-framework-logging`), plus the supporting bootstrap call in `contexter_server/__init__.py` and the new/extended test suites. Iter-2 state (zero findings) re-verified against the current working tree (HEAD 27e031d + uncommitted changes).

**Verdict:** CONDITIONAL PASS (class: 0 Critical / 0 High / 0 Medium / 1 Low)

2026-08-02 · 1 finding · Security Architect

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 1 |

> **Security Scope**
> Threat-modeled the iter-3 changed surface: (1) `contexter-core/src/storage/rocksdb.rs` `count_sessions` estimate fast path — panic paths, unbounded work, injection surface, parse-failure fallback, filter-guard completeness vs `SessionFilter` fields, estimate-vs-exact divergence (deletes/tombstones); (2) `contexter-server/src/contexter_server/fastmcp_logging.py` — filter can only drop records (no logging widening), prefix-matching safety (attacker-controlled content cannot reach framework loggers), filter-placement semantics (originating-logger-only), idempotency, DoS via filter logic, survival of FastMCP `configure_logging` (verified against installed framework source); (3) `__init__.py` bootstrap — import-time side effects, no secrets; (4) test files — content-leak assertions (EC-FL-004), byte-identical client frames (REQ-FL-002), diagnostics retention (REQ-FL-003), auth matrix (EC-FL-003); (5) full-scope re-verification of iter-2 state: no secrets in logs, bounded id handling, auth matrix intact, CONTEXTER_ env canonicalization, constant-time key compare (`hmac.compare_digest`), no unbounded input.

---

## 02 · Vulnerability Findings

## 1 finding — 1 Low (defense-in-depth gap, currently unreachable)

### F-IT3-01 — LOW — FastMCP failure-stderr filter coverage is incomplete vs. its documented scope (prompts / sampling namespaces not covered)

**Location:** `contexter-server/src/contexter_server/fastmcp_logging.py` — `_EMITTER_LOGGERS` (L47-51) and `_FRAMEWORK_ERROR_PREFIXES` (L31-35)

**Issue:** The module docstring states the filter suppresses "any tool/resource/prompt error" and the prefix tuple includes `"Error rendering prompt "`, but the framework emitters for those record classes are NOT covered:

1. `fastmcp/prompts/function_prompt.py:370` runs `logger.exception(f"Error rendering prompt {self.name}")` on logger `fastmcp.prompts.function_prompt` — not in `_EMITTER_LOGGERS`. Python's `logging` applies ONLY the originating logger's filters (`Logger.handle` → `self.filter`), so a failing prompt render would still emit a rich traceback box on stderr (both from `function_prompt.py` and the covered `server.py` site — the uncovered one would leak the box).
2. `fastmcp/server/sampling/run.py` runs `logger.exception(f"Error calling sampling tool '{tool_use.name}'")` on logger `fastmcp.server.sampling.run` — not in `_EMITTER_LOGGERS`, AND the message prefix `"Error calling sampling tool "` does not match `_FRAMEWORK_ERROR_PREFIXES` (`"Error calling tool "` — "sampling" breaks the prefix). Even if the logger were added to `_EMITTER_LOGGERS`, the record would still pass the filter.

**Exploitability today: none.** Contexter registers zero prompts and zero sampling handlers (`grep -rn "add_prompt|@mcp.prompt"` over `contexter-server/src/` is empty; no sampling usage in the server). The 9 iter-2 error classes (tool/resource/validation/auth/engine/not-found/storage/launch) all emit from `fastmcp.server.server` — covered. AC-FL-001/002 hold for the current server surface; the 13 new EFS tests pass.

**Security risk:** Latent regressions of REQ-FL-001 (2672-char traceback box on stderr for prompt/sampling failures) if a future feature registers prompts or invokes sampling. A rich traceback box can carry source frames and partially wrapped payload content to stderr — the exact disclosure vector this contract exists to close. Defense-in-depth gap only; no current reachable exposure.

**Recommendation (Should Fix):** Extend `_EMITTER_LOGGERS` with `"fastmcp.prompts.function_prompt"` and `"fastmcp.server.sampling.run"`, and add `"Error calling sampling tool "` to `_FRAMEWORK_ERROR_PREFIXES`. Consider prefix matching by word-boundary/regex on the emitting namespaces, or asserting the complete emitter set at test time by scanning the installed `fastmcp` package for `logger.exception` sites containing "Error calling/reading/rendering".

### Re-verified — no re-statements

- **F-IT1-01 id echo bounded** — unchanged from iter-2; `_bounded` at six call sites; id-bounding suite green. No new unbounded-echo surface in iter-3 changes.
- **F-IT1-02 handler log bindings bounded** — unchanged; no new log bindings introduced in iter-3 source changes.
- **F-IT1-03 resource URI `_api_key` / SSE gating** — unchanged; docs contract met (iter-2).
- **F-IT1-04 camelize collision invariant** — unchanged; invariant suite green (iter-2).
- **F-IT1-05 CONTEXTER_MAX_REQUEST_BODY canonical** — unchanged; new iter-3 tests use only canonical `CONTEXTER_LOG_FILE` / `CONTEXTER_API_KEY` / `CONTEXTER_MAX_REQUEST_BODY`; no bare-name env reads in iter-3 additions.
- **Iter-2 new code** — count endpoints, `total=-1` sentinel, bridge diagnostics logging, launcher RuntimeError pin: unchanged and re-verified in the working tree (count_agents/count_skills estimate semantics identical to the new count_sessions path).

### Iter-3 code — detailed checks (no findings)

- **`count_sessions` fast path (rocksdb.rs:715-731):** `property_value_cf(...)` result handled with `.ok().flatten()` — both error and `None` fall through to the exact scan; `val.parse::<u64>()` uses `if let Ok` — parse failure falls through; NO panic/unwrap/expect on any path. Property name is a hard-coded constant string — no injection surface. Work is O(1) (CF property read); the fallback is the same full scan that previously ran for every call — no new unbounded work. Fast-path guard (`agent_id.is_none() && status.is_none()`, after `project.is_some()` returns via index-prefix scan) is complete: `SessionFilter` has only project/agent_id/status/limit/offset and limit/offset do not affect counts, so the estimate fires exactly on the fully-unfiltered case. Filtered counts remain exact scans (REQ-CS-002). `estimate-num-keys` may over-count after `delete_session` until compaction (tombstones) — documented accepted semantics (EC-CS-003, `test_bridge_live_coverage.py` docstring, `session_test.rs` comment), identical precedent accepted for count_agents/count_skills in iter-2; session counts are informational analytics, not an authorization input — not a security issue.
- **`fastmcp_logging.py` filter mechanics:** the filter only returns `False` (drop) — it cannot cause or widen output anywhere; no log record is ever re-emitted. `record.getMessage().startswith(...)` on three short constant prefixes is O(prefix) with no allocation beyond `getMessage()` — no DoS surface. Framework error-call records are f-strings without `%`-args, so `getMessage()` cannot raise on mismatched args; attacker-controlled content never reaches `fastmcp.*` logger records (tool/resource/prompt names are server-registered constants, and the record `msg` is framework-generated). Installation is idempotent (`_INSTALLED_ATTR` guard) and the filter object is stateless — thread-safe for concurrent tool calls (EC-FL-005). Filter placement is correct per stdlib semantics: installed on the emitting logger `fastmcp.server.server` (and ancestors as belt-and-braces), which is the only place Python applies filters. Verified against installed framework source: `fastmcp.utilities.logging.configure_logging` removes handlers only and never touches `logger.filters`, so the iter-3 assumption that filters survive framework reconfiguration is correct.
- **`__init__.py` bootstrap:** import-time call is idempotent, performs no I/O, touches only stdlib logger objects — no secrets, no startup cost beyond logger creation; every entry point (run_mcp.py, API app, tests) gets the policy deterministically.
- **Test files:** `test_framework_efs_stderr.py` (13 tests) asserts stderr ≤512 chars AND ≤512 bytes, no box chars, no `Traceback`, no `File "` frames, byte-identical client frames for all 7 error classes (REQ-FL-002 — no success smuggling), no 10KB query content leak to stderr (EC-FL-004), per-failure bound under 5-way concurrency (EC-FL-005), full traceback retained in `CONTEXTER_LOG_FILE` (AC-FL-003), filter presence on the `fastmcp` namespace (EC-FL-001), and unit-level filter pass/drop behavior. `agent_skill_test.rs` adds count parity tests (unfiltered matches store, empty → 0, project-filtered exact) — no security surface, but they pin the estimate-vs-exact semantics on fresh stores. `session_test.rs` correctly swaps the concurrency invariant to the exact `list_sessions` scan (the estimate can lag deletes — a correctness adaptation, not a security change). `test_bridge_live_coverage.py` verifies 12-session store → `count_sessions()==12` and `get_overview().total_sessions==12` end-to-end. No secrets, fake-key constants only (`test-key-123`).

---

## 03 · Security-Critical Code Highlights

- **Constant-time auth intact:** `hmac.compare_digest` in `mcp_tools/auth.py:56` and `api/deps.py:64`; new EFS tests assert byte-identical auth-missing/auth-wrong frames (EC-FL-003 serialization survived unchanged).
- **No secrets in new paths:** `fastmcp_logging.py` contains no env reads, no credentials, no content; it only installs stateless filters. New tests use canonical `CONTEXTER_*` envs and fake keys. The filter drops records — it cannot leak anything it does not already receive, and the framework error records it drops contain only tool/resource/prompt names (server-registered).
- **No logging widening:** the filter is scoped to `fastmcp.*` loggers; structlog/root-logger output (bridge `handler_error`, `bridge_call_failed`) is untouched — suppression cannot mask contexter's own bounded security-relevant lines.
- **Error containment preserved:** client-visible `isError` frames byte-identical (asserted); full tracebacks retained server-side in the diagnostics log (AC-FL-003 asserted); stdout remains pure JSON-RPC (AC-FL-005).
- **Input validation unchanged and effective:** content ≤1 MB, query ≤10k, limit clamps; oversized-query failure emits no content to stderr (tested).
- **Full suite:** baseline 867 passed + new Rust count tests + 13 framework-EFS tests, 0 failures (per Worker Handoff Report; live coverage green against rebuilt wheel).

---

## 04 · Remediation Recommendations

> **Must Fix**
> None — 0 Critical / 0 High / 0 Medium.

> **Should Fix**
> F-IT3-01 (Low): add `"fastmcp.prompts.function_prompt"` and `"fastmcp.server.sampling.run"` to `_EMITTER_LOGGERS` and `"Error calling sampling tool "` to `_FRAMEWORK_ERROR_PREFIXES` (or word-boundary matching), so any future prompt/sampling feature cannot regress REQ-FL-001. Optionally add a test that enumerates installed-framework `logger.exception` sites for the three error-call prefixes and asserts each emitter logger carries the filter.

> **Consider**
> None beyond F-IT3-01.

---

_Generated by Security Architect · 2026-08-02 · Validation Contract: 2026-08-01-mcp-live-fix_
