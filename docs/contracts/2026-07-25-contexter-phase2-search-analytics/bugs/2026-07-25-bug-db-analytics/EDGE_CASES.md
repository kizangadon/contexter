# Edge Cases — Bug-DB-Analytics

- EC-01: Empty params slice is passed — should work fine (no params to bind)
- EC-02: StorageBackend is None — return error, don't panic
- EC-03: Column family has 0 entries in RocksDB — sync succeeds, table is empty
- EC-04: Column family doesn't exist in RocksDB — return ColumnFamilyNotFound error
- EC-05: Concurrent sync and query — thread-safe via existing Mutex
- EC-06: StorageBackend disconnected mid-sync — return error, don't leave table in inconsistent state
