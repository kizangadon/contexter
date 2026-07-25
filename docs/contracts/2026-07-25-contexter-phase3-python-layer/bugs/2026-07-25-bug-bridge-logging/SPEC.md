# Bug: Bridge Call Logging + Error Logging

**Sources:** SPEC REQ-OBS-002, REQ-OBS-003 (partial), Code Reviewer P2 (findings 2, 3)

**Files:** `core/bridge.py`, `services/correlation_service.py`, `services/onboarding_service.py`

**Problems:**
1. Bridge `_run()` method does not log function calls — no function name, args summary, or duration logged (REQ-OBS-002)
2. Error logging is not systematic across bridge/service layers (REQ-OBS-003)
3. `correlation_service.py` has bare `except Exception: pass` (line 55-56)
4. `onboarding_service.py` has bare `except Exception: return 0` (lines 43-48, 50-55)

**Fix:**
1. Add structlog logging to `_run()` — log function name, args summary (truncated), and duration
2. Log errors before propagating (bridge), and add systematic error logging
3. Replace bare `except Exception: pass` with logged exceptions
4. Replace bare `except Exception: return 0` with logged exceptions

**Acceptance:** Bridge calls produce structured logs. Exceptions are logged with traceback. Correlation/onboarding errors are logged, not silently swallowed.
