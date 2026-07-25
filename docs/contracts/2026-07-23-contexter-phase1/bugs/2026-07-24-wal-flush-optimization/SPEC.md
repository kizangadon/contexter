# Bug 8: WAL Flush Optimization (Perf H1, H5)

## Problem
Every mutating operation calls `flush_wal(true)`, causing an fsync syscall (1-10ms) per write. Additionally, the SharedBackend RwLock adds serialization overhead.

## Fix Requirements
1. Add `RocksDbConfig.wal_sync` boolean field (default: true) controlling whether `flush_wal` is called after each write
2. When `wal_sync = false`, skip `flush_wal` calls for write throughput
3. Verify SharedBackend RwLock correctness is maintained
