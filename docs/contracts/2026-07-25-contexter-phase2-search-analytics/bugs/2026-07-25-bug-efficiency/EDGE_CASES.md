# Edge Cases — Bug-Efficiency

- EC-01: efficiency_map CF is empty in RocksDB — cache is empty, queries return empty results
- EC-02: CF doesn't exist — return error, don't panic
- EC-03: Cache TTL = 0 — every call skips cache (always fresh)
- EC-04: Cache has stale entry but table's last_sync is older — stale entry still valid until TTL
- EC-05: Concurrent cache read and write — use `Arc<Mutex<...>>` or `RwLock`
