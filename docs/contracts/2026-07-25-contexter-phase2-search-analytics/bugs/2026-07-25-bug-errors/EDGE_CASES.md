# Edge Cases — Bug-Errors

- EC-01: Mutex is poisoned during read — recover and return stale/dummy data
- EC-02: Temp file cleanup fails (file locked) — log warning, don't crash
- EC-03: Multiple errors in a pipeline — propagate the first error, don't swallow
