# Acceptance Criteria

### AC-01: Empty created_at handling
GIVEN a RocksDB record with empty or missing `created_at` field  
WHEN analytics sync processes it  
THEN the record MUST be skipped with a logged warning, not crash or silently pass an empty string
