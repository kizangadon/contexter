# Acceptance Criteria — Bug-Errors

- AC-01: No bare `unwrap()` calls remain in engine source files (search.rs, analytics.rs, memory.rs, mod.rs)
- AC-02: `EngineError::UnsupportedOperation(String)` added and used
- AC-03: Mutex poisoning recovers (doesn't panic)
- AC-04: Temp directories created during analytics are cleaned up on drop
- AC-05: All existing tests continue to pass
