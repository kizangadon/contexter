# Bug: Sequential Independent Bridge Calls

**Sources:** Performance H1, Code Reviewer P3 #9

**Files:** `services/analytics_service.py`, `services/export_service.py`, `services/memory_service.py`, `services/search_service.py`, `services/onboarding_service.py`

**Problem:** Multiple services make independent sequential bridge calls that could run in parallel:
- AnalyticsService.get_overview/health/resources: 3 sequential calls → can be asyncio.gather()
- ExportService.submit: 4 sequential list/search calls → can be asyncio.gather()
- MemoryService.search: search_memories + count_memories → asyncio.gather()
- SearchService.search: search_memories + optional list_sessions → asyncio.gather()
- OnboardingService.get_progress: 3 sequential calls → asyncio.gather()
- OnboardingService.submit_wizard: N sequential set_settings → asyncio.gather()

**Fix:** Replace sequential `await` calls with `asyncio.gather()` for independent bridge operations. Maintain error handling (one failure should not cancel others, or handle appropriately).

**Acceptance:** Analytics endpoints resolve in ~1/3 the wall-clock time. Export resolves in ~1/4 time. All tests pass with mocked awaitable calls.
