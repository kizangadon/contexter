# Acceptance Criteria

### AC-01: 0o700 permission verified
GIVEN an Engine is opened at a new path  
WHEN the storage directory permissions are checked  
THEN on Unix, the permissions MUST include owner-read/write/execute and exclude group/other access
