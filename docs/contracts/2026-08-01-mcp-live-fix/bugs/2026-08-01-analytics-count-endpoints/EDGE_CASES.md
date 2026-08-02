# EDGE_CASES — Analytics Count Endpoints
- EC-ACE-001: engine error during count → `_safe_get` degrades to 0 with warning (existing behavior).
- EC-ACE-002: count endpoints return same numbers as list-based count for >0 stores (parity test).
- EC-ACE-003: CLI status path (which uses get_overview) still renders correctly.
- EC-ACE-004: bridge mock guard (`_SYNC_ENGINE_CLASS`) applies to new methods.
