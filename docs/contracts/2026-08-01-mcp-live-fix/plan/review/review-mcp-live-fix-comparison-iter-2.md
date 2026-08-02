# Comparison Report — MCP Server Live-Functionality Repair (Iteration 2)

> Auto Bug Loop Iteration 2 · User-Testing Validator · 2026-08-02
> **Companion to:** `review-mcp-live-fix-user-testing-review-iter-2.md` (25/26 AC pass)
> **Trigger:** AC-EFS-001 FAIL — total engine-failure stderr exceeds the ≤512-char/no-raw-traceback letter end-to-end.

---

## 1 · What Was Compared

| Contract | Requirement | Measured (live, real engine) | Status |
|---|---|---|---|
| AC-EFS-001 / REQ-EFS-002 | Mid-call engine failure → stderr shows **≤512 chars total for that failure** and **no raw traceback** | **2897 bytes** total for one `get_session` engine failure | ❌ FAIL |
| REQ-EFS-001 (bridge layer) | Bridge emits a single concise structured stderr line; full exception → bounded diagnostics channel | Bridge emits **one 224-char line**; full traceback → launch log file (3046 bytes) | ✅ PASS (bridge portion) |
| REQ-EFS-003 / AC-EFS-003 | stdout stays pure JSON | 5/5 JSON-RPC frames, ids [1,2,3,4,5], 0 bad lines | ✅ PASS |

## 2 · The Gap (Bridge vs Framework)

The iter-2 fix **correctly** replaced the bridge's `logger.exception('bridge_call_failed')` with a single structured `logger.error` line. Verified live:

```
2026-08-02T06:09:49.888921Z [error    ] bridge_call_failed [contexter_server.core.bridge]
  args_summary="('not-a-uuid',)" diagnostics_log=/tmp/opencode/iter2-launch.log
  exception_type=ValueError method=get_session          ← 224 chars, one line
```

However, **FastMCP's own framework logger** (`fastmcp.*`, `propagate=False`, `RichHandler(console=Console(stderr=True))`) is **not configured by the feature** (`contexter_server/__init__.py` sets only the root stdlib logger to INFO; `run_mcp.py` sets no fastmcp log level). Its generic `except Exception` handler at `fastmcp/server/server.py:1297` runs `logger.exception(f"Error calling tool {name!r}")` for **every** tool error — including this engine failure — rendering a rich box-drawing traceback (2672 chars, with source frames) on stderr:

```
                    Error calling tool 'get_session'
                    ╭─────────── Traceback (most recent call last) ────────────╮
                    │ /home/don/.local/lib/python3.12/site-packages/fastmcp/se │
                    │ rver/server.py:1282 in call_tool                         │
                    │ ... 5 frames hidden ...                                  │
                    │ /home/don/Code/contexter/contexter-server/src/contexter_ │
                    │ server/core/bridge.py:231 in _run                        │
                    │ ❱ 231 │ result = await loop.run_in_executor(...)         │
                    ╰──────────────────────────────────────────────────────────╯
                    ValueError: invalid session id "not-a-uuid": invalid
                    character: found `n` at 0
```

**Measured split (one engine failure):** bridge line 224 chars + FastMCP box 2672 chars = **2897 bytes total**.
**Framework boxes observed:** 9 across the live session (validation, auth, engine error classes) — framework-wide, not engine-specific.

## 3 · Why This Is a Finding (not out of scope)

1. **AC-EFS-001 is written about observable stderr for the failure** ("WHEN server runs, THEN stderr shows ≤512 chars total for that failure") — no layer qualifier. A user watching stderr sees the raw traceback box.
2. **The server controls this surface.** Two proven paths exist in the running stack:
   - Configure the `fastmcp` logger (level/filter) at bootstrap, or
   - Raise `FastMCPError` subclasses for handler errors — `fastmcp/server/server.py:1284-1287` logs `FastMCPError` with `exc_info=False` (no traceback box). `HandlerError`/`MCPAuthError` are currently `ValueError` subclasses (errors.py:24, auth.py:15), so they fall into the generic `except Exception` path.
3. **No data/functionality impact** — stdout pure, no content leak, diagnostics channel correct, client-visible error frames unaffected. Severity: **LOW** (contract letter violation, operator-facing stderr hygiene).

## 4 · Evidence Artifacts

| Artifact | Content |
|---|---|
| `/tmp/opencode/iter2-live-stderr-clean.txt` | Full live stderr capture: bridge lines + 9 FastMCP framework boxes |
| `/tmp/opencode/iter2-launch.log` | Diagnostics log: full tracebacks for every `bridge_call_failed` event (3046 bytes) |
| `test_bridge_engine_failure_stderr.py` (13 passed) | Unit-level bounded-stderr regression tests (bridge logger scope) — pass, but do not exercise the FastMCP framework logger path |
| Full suite `-W error::pydantic.warnings.UnsupportedFieldAttributeWarning` | 867 passed, 0 failures, 0 pydantic warnings |

## 5 · Suggested Resolution (for next Auto Bug Loop contract)

- **Option A (minimal):** configure `logging.getLogger("fastmcp")` level/filter at server bootstrap so framework `logger.exception` boxes are suppressed or downgraded.
- **Option B (contract-aligned):** have `HandlerError`/`MCPAuthError` subclass `fastmcp.exceptions.FastMCPError` with an appropriate `log_level` so FastMCP logs `exc_info=False` (no traceback) while keeping the client-visible structured error frames unchanged.
- **Add regression:** extend EFS tests to assert the FastMCP-framework stderr path (e.g., a test that runs a tool error through `mcp.call_tool` and asserts stderr contains no `╭ Traceback` box).

---

_Generated by User-Testing Validator · 2026-08-02 · Validation Contract: 2026-08-01-mcp-live-fix · Iteration 2_
