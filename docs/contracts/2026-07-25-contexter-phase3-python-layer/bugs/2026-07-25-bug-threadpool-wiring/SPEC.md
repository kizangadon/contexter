# Bug: ThreadPoolExecutor Not Wired to asyncio.to_thread()

**Sources:** SPEC REQ-BRG-003 (partial), Performance M2

**File:** `core/bridge.py` lines 29-51

**Problem:** `ThreadPoolExecutor(max_workers=4)` is created and stored as `self._pool` (line 33) but never passed to `asyncio.to_thread()`. Python's `to_thread()` uses the default loop executor, so the `max_workers=4` constraint is not enforced. The pool is dead code.

**Fix:** Pass `self._pool` to `asyncio.to_thread()` calls using the `loop=` executor parameter pattern, or use `loop.run_in_executor(self._pool, fn)` instead of `asyncio.to_thread(fn)`. Recommend `loop.run_in_executor(self._pool, fn)` for explicit executor control.

**Acceptance:** Bridge uses the configured 4-worker ThreadPoolExecutor, not the default executor.
