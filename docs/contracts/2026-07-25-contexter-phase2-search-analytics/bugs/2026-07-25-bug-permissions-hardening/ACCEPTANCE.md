# Acceptance Criteria

## Given-When-Then

### AC-01: TempDirGuard sets 0o700
GIVEN a `TempDirGuard` is created  
WHEN the temporary directory is inspected  
THEN its permissions MUST be `0o700` (owner-only access)

### AC-02: Tantivy index dir sets 0o700
GIVEN a `TantivyIndex` is opened  
WHEN the index directory is inspected  
THEN its permissions MUST be `0o700`

### AC-03: Snapshot file sets 0o600
GIVEN `save_snapshot_data()` is called  
WHEN the output file is inspected  
THEN its permissions MUST be `0o600` (owner read/write only)

### AC-04: Read-only test updated
GIVEN the `test_read_only_path_error` test  
WHEN run  
THEN it MUST pass without expecting an error from the read-only dir setup
