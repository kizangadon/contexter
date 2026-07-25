# Bug 4: Telemetry + Python Async Wrapper + Test Gaps

## Problem
Engine has no telemetry event recording, Python async wrapper with ThreadPoolExecutor doesn't exist, and test gaps: no read-only path error test, WAL recovery test, or key encoding correctness test.

## Root Cause
Telemetry recording was specified in design preview but not implemented. Python async wrapper was specified in spec but not created. Tests were incomplete.

## Fix Requirements
1. Create `python/core_bridge.py` — async wrapper using `asyncio.to_thread()` + `ThreadPoolExecutor(max_workers=4)`
2. Add `test_read_only_path_error` to integration tests
3. All existing tests must still pass
