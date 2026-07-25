# Bug: Large Content Threshold Uses String Length

**Sources:** Performance H4, edge case E-003/E-004 implication

**File:** `core/bridge.py` lines 84-95, 107-114

**Problem:** `len(content) >= _LARGE_CONTENT_THRESHOLD` (102400) counts Unicode characters, not bytes. Multi-byte UTF-8 content (emoji, CJK) with char-length < 100K can exceed 100KB in bytes, bypassing the PyBytes path.

**Fix:** Change threshold check to `len(content.encode("utf-8")) >= _LARGE_CONTENT_THRESHOLD` in both `create_memory` and `update_memory`.

**Acceptance:** Content with fewer than 100K chars but exceeding 100KB of UTF-8 bytes correctly triggers the PyBytes path. Update existing test to use byte-length assertion.
