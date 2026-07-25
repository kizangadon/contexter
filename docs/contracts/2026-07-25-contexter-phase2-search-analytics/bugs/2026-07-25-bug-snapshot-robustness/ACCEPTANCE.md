# Acceptance Criteria

### AC-01: Max-length guard
GIVEN a snapshot with a length prefix > 1024 bytes  
WHEN `read_string()` is called  
THEN it MUST return an error, not allocate the buffer

### AC-02: Strict UTF-8
GIVEN a snapshot with non-UTF-8 bytes in a string field  
WHEN `read_string()` is called  
THEN it MUST return an error, not silently replace characters

### AC-03: TOCTOU eliminated
GIVEN `load_snapshot()` is called  
WHEN the file metadata is checked  
THEN it MUST be checked on the opened `File` handle, not on the path before opening
