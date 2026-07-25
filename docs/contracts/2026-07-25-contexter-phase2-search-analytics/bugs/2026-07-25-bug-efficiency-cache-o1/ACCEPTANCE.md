# Acceptance Criteria

### AC-01: Only requested entry checked
GIVEN an efficiency cache with N entries  
WHEN `get_efficiency_scores("session-X")` is called  
THEN only entry "session-X"'s TTL MUST be checked (not all N entries)
