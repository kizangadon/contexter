# Acceptance Criteria

### AC-01: Per-entry TTL check
GIVEN an efficiency cache with N entries  
WHEN `get_efficiency_scores()` is called for one session  
THEN only that session's TTL MUST be checked (not all N entries)
