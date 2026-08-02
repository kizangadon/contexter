# Preview — Pre-Existing Lifespan Test Fix

## Approach
```mermaid
flowchart LR
  A[test_lifespan_shutdown_joins_thread] --> B[unique temp data dir per test]
  B --> C[no RocksDB LOCK contention]
  C --> D[clean join on shutdown]
```
Isolate engine data dirs in tests; preserve original shutdown-join intent; suite fully green.

## Fix boundary
Lifespan test file (test infra only).

## Acceptance mapping
AC-LS-001..003, EC-LS-001..003.
