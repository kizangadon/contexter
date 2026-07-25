# Acceptance Criteria

### AC-01: QueryParser cached
GIVEN two consecutive `search()` calls  
WHEN the second call executes  
THEN it MUST reuse the QueryParser from the first call (not construct a new one)
