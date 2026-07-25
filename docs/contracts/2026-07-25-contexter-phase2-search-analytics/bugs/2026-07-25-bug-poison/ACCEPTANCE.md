# Acceptance Criteria — Bug-Poison

- AC-01: If a thread panics while holding DuckDbEngine's Mutex, subsequent `.lock()` calls recover
- AC-02: Same for Engine's RwLock/L1/L2/L3/L5 locks
- AC-03: All existing tests continue to pass
