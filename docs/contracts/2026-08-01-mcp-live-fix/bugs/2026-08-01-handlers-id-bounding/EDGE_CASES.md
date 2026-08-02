# EDGE_CASES — Handler ID Bounding
- EC-HIB-001: id is None → no crash, bounded path handles None.
- EC-HIB-002: id is empty string → bounded fine.
- EC-HIB-003: id exactly 64 chars → unchanged (boundary).
- EC-HIB-004: id 65 chars → truncated to 64.
- EC-HIB-005: non-string id (int) → coerced/bounded without crash (existing handler types unchanged).
