# Bug: File Security — Permissions and TOCTOU

**Severity:** MEDIUM  
**Root Cause:** Analytics temp files and RocksDB directory have default permissions; snapshot load has potential TOCTOU race.

## Requirements

### REQ-FIX-001: Set restrictive permissions on temp files
When creating temp directories for analytics, set `umask` or explicitly `chmod` to `0o700` to prevent other users from reading temp data.

### REQ-FIX-002: Verify snapshot file before load
Add a stat + file size check before loading snapshot to verify the file hasn't been truncated or replaced between the stat and the actual open (TOCTOU mitigation).
