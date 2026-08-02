# SPEC — FastMCP Framework Logging: Bounded Failure Stderr (End-to-End)

> Parent contract: `2026-08-01-mcp-live-fix` · Auto Bug Loop Iteration 3
> Source finding: **AC-EFS-001** (User-Testing Validator, `plan/review/review-mcp-live-fix-user-testing-review-iter-2.md` §02/§04; `plan/review/review-mcp-live-fix-comparison-iter-2.md`)
> Verdict: CONDITIONAL PASS — 25/26 AC; 1 LOW contract violation at framework level

## Problem

AC-EFS-001 requires: engine failure → stderr shows **≤512 chars total for that failure** and **no raw traceback**. The bridge layer fix (iter-1) is verified correct: ONE concise 224-char structured line, full traceback only in the diagnostics log file.

However, FastMCP's **framework logger** (`fastmcp.*`, `propagate=False`, `RichHandler(console=Console(stderr=True))`) is NOT configured by the feature (`contexter_server/__init__.py` sets only the root stdlib logger to INFO; `run_mcp.py` sets no fastmcp log level). Its generic `except Exception` handler at `fastmcp/server/server.py:1297` runs `logger.exception(f"Error calling tool {name!r}")` for EVERY tool error, rendering a 2672-char rich traceback box (with source frames) on stderr.

Measured (live): bridge line 224 chars + FastMCP box 2672 chars = **2897 bytes** for one engine failure — exceeds AC-EFS-001's ≤512-char letter. 9 framework boxes observed across error classes (validation, auth, engine).

## Requirements

### REQ-FL-001 — Bounded total failure stderr
The feature SHALL configure/route the `fastmcp` framework logger so that for any tool/resource error (engine, validation, auth), the TOTAL stderr emitted for that failure is ≤512 chars and contains NO raw traceback (no `Traceback`, no source frames, no rich box drawing).

### REQ-FL-002 — Client-visible error frames unchanged
Structured `isError` responses, `MCPAuthError` messages (`'API key required...'` / `'Invalid API key.'`), and handler error messages SHALL be byte-identical to current behavior. This is stderr hygiene only.

### REQ-FL-003 — Diagnostics channel unchanged
Full tracebacks SHALL continue to be persisted to the diagnostics log file (bridge `_write_runtime_failure_diagnostics`, CONTEXTER_LOG_FILE / `~/.contexter/logs/mcp-launch.log`). No loss of debuggability.

### REQ-FL-004 — Success path and stdout untouched
Success-path stderr (INFO lifecycle only at default level), DEBUG per-call logs, stdout JSON-RPC purity, and launch behavior (rc=2 on corrupt engine) SHALL be unchanged.

### REQ-FL-005 — Tests
Regression tests SHALL exercise the FastMCP-framework stderr path: running tool calls that raise engine/validation/auth errors through the FastMCP call path and asserting stderr contains no traceback box and stays ≤512 chars per failure (extend the existing `test_bridge_engine_failure_stderr.py` suite or add a framework-level EFS test module).

## Implementation Options (Worker decides, both proven)

- **Option A (minimal):** configure `logging.getLogger("fastmcp")` level/filter at server bootstrap (run_mcp.py or `contexter_server/__init__.py`) so framework `logger.exception` boxes are suppressed/downgraded. NOTE: `propagate=False` means root-logger configuration does NOT reach the fastmcp namespace — the `fastmcp` logger must be configured directly.
- **Option B (contract-aligned):** have `HandlerError`/`MCPAuthError` subclass `fastmcp.exceptions.FastMCPError` with an appropriate `log_level` so FastMCP logs `exc_info=False` (fastmcp/server/server.py:1284-1287) — no traceback box — while client-visible error frames stay unchanged. Verify MCPAuthError serialization (`MCPAuthError` message mapping in handlers/auth) survives subclassing.

Either option MUST satisfy all five requirements, verified by tests.

## Non-Goals

- No change to FastMCP framework source files (site-packages is read-only reference).
- No change to bridge stderr line format (already correct).
- No change to client-visible error message text.

## Artifacts

- Contract dir: `docs/contracts/2026-08-01-mcp-live-fix/bugs/2026-08-01-fastmcp-framework-logging/`
- References: `plan/review/review-mcp-live-fix-comparison-iter-2.md` (gap analysis, suggested resolution §5), `plan/review/review-mcp-live-fix-user-testing-review-iter-2.md` (§04 finding 1), evidence `/tmp/opencode/iter2-live-stderr-clean.txt`, `/tmp/opencode/iter2-launch.log`
- Framework reference (read-only): `fastmcp/server/server.py:1284-1287` (FastMCPError path, `exc_info=False`) and `:1297` (`logger.exception` generic path)
