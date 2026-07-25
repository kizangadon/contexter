# Bug: Shadowing Built-ins + Code Quality Nits

**Sources:** Code Reviewer P2 #4, P3 #7, #8, #10, #12, #13, #14, #15, #16, #17, Security LOW-01, INF-01

**Files:** `mcp_tools/handlers.py`, `main.py`, `api/deps.py`, `api/efficiency.py`, `api/files.py`, `api/changelog.py`, `api/correlation.py`, `cli/session_commands.py`, `tests/core/test_bridge.py`, `pyproject.toml`

**Problems:**
1. `type` shadows built-in in MCP handlers (handlers.py:60,142) → rename to `type_filter` or `entity_type`
2. `_run_mcp_server` references `logger` before definition (main.py) → move logger up
3. `_get_service` in deps.py returns `Any` → use TypeVar
4. ExportService uses magic key suffix `f"{id}_data"` → separate dict or dataclass  
5. `compute_efficiency` returns hardcoded 1.0 → either implement or remove
6. Several API endpoints return TODO stubs → add `deprecated` marker or remove from router
7. `handle_store_memory` doesn't catch ValueError from UUID() parse → add try/except
8. Settings routes missing `response_model` → add model
9. `_format_session` in CLI accesses `.name` without null check → normalize
10. Fragile test assertion in test_bridge.py:219 → simplify
11. httpx dependency unused → remove or move to optional
12. AnalyticsService._status_cache dead code → remove

**Acceptance:** No built-in shadowing. Logger declarations are conventional. deps.py has typed return. No dead code. Test assertions are robust. Unused dependency removed.
