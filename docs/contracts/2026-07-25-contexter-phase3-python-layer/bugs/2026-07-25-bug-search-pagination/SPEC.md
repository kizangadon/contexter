# Bug: Search/List Endpoints Lack Pagination

**Sources:** Performance H3, Code Reviewer P3 #9 (related)

**Files:** `services/search_service.py`, `services/memory_service.py`, `core/bridge.py`

**Problem:** 
1. `search_memories({})` returns ALL results — no limit/offset on Rust bridge
2. `MemoryService.list()` calls `search_memories({})` with empty query — no pagination
3. `SearchService.search()` fetches all results then takes a slice

**Fix:**
1. Add `limit` and `offset` parameters to bridge's `search_memories`, `list_sessions`, `list_agents`, `list_skills` methods
2. Add default limit (100) to MemoryService.list()
3. Ensure SearchService passes limit/offset to the bridge

**Acceptance:** Bridge methods accept limit/offset. List endpoints respect pagination. Tests verify pagination parameters.
