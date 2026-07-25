# Bug: SettingsService Blocks Event Loop with Sync I/O

**Sources:** Performance H2

**File:** `services/settings_service.py`

**Problem:** `yaml.safe_load()` (file read + YAML parse) and `yaml.dump()` + file write run synchronously in async methods, blocking the event loop for all concurrent requests.

**Fix:** Offload YAML file I/O via `asyncio.to_thread()` or `loop.run_in_executor()`. Wrap both `yaml.safe_load()` + file read and `yaml.dump()` + file write.

**Acceptance:** SettingsService I/O operations do not block the event loop. Tests verify thread pool usage.
