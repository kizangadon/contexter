# EDGE CASES — FastMCP Framework Logging: Bounded Failure Stderr (End-to-End)

## EC-FL-001 — `propagate=False` on fastmcp namespace
FastMCP loggers have `propagate=False` — configuring the ROOT logger does NOT reach `fastmcp.*`. Any level/filter configuration MUST target the `fastmcp` logger namespace directly (or the specific child logger). A fix that only touches the root logger will not work — verified by a test that asserts the framework path is actually silenced.

## EC-FL-002 — Framework `logger.exception` on generic `except Exception`
`HandlerError`/`MCPAuthError` are currently `ValueError` subclasses → they fall into FastMCP's generic `except Exception` path (server.py:1297) which logs with `exc_info=True` (traceback box). If Option B (FastMCPError subclass) is chosen, ensure ALL error classes used by handlers route through the FastMCPError path — any error class that misses the subclass (e.g., a raw ValueError raised from a handler) will still render a box.

## EC-FL-003 — MCPAuthError serialization after subclassing
`MCPAuthError` message serialization (`'API key required...'` / `'Invalid API key.'`) must survive subclassing FastMCPError — verify with existing auth tests (`tests/mcp/test_mcp_auth.py`, AC-ES-003) plus a live probe.

## EC-FL-004 — No content/secret leak
Bounded stderr must never truncate INTO sensitive data in a way that leaks it — the args_summary capping (≤64 chars) and id bounding already handle this; the framework path must not re-print full arguments. Verify a 10KB-content failure produces no content on stderr (mirror AC-BH-001).

## EC-FL-005 — Concurrency (multiple simultaneous failures)
FastMCP handles concurrent tool calls; concurrent failures must each emit ≤512 chars total (no interleaving that combines into a giant stderr block). Verify with the existing concurrency tests (test_protocol_edge_cases.py).

## EC-FL-006 — FastMCPError path already quiet
If FastMCPError subclasses are used, note server.py:1284-1287 already logs FastMCPError with `exc_info=False` — confirm no double logging with the bridge's `bridge_call_failed` ERROR line (bridge logs once, framework logs without traceback, total ≤512).

## EC-FL-007 — Regression scope
The EFS regression tests must run at the FastMCP call-path level (not just the bridge logger scope) — the iter-2 gap was precisely that `test_bridge_engine_failure_stderr.py` (13 tests) covered the bridge but not the framework logger. A test that runs a tool error through FastMCP's call path and asserts no `╭ Traceback` box on stderr closes the gap.
