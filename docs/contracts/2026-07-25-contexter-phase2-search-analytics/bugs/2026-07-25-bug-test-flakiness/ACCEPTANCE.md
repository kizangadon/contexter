# Acceptance Criteria

### AC-01: Unique temp dir per instance
GIVEN two `TempDirGuard` instances created in parallel  
WHEN their temp directory paths are compared  
THEN they MUST be different (no collision)

### AC-02: Cleanup on drop
GIVEN a `TempDirGuard` is dropped  
WHEN the temp directory is inspected  
THEN it MUST be removed (cleaned up)
