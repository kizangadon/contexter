# Bug: In-Memory State in Export/Notification Services

**Sources:** Code Reviewer P2 #5, Performance M1, Security MED-05

**Files:** `services/export_service.py`, `services/notification_service.py`

**Problem:** Both `ExportService` and `NotificationService` store all data in instance-level dicts (`self._exports`, `self._notifications`) despite accepting a `StorageEngine` in their constructor. Data is lost on restart. No eviction policy exists.

**Fix:**
1. Store export statuses via the bridge's `set_setting`/`get_setting` or bridge dedicated method
2. Store notifications via bridge storage
3. Add LRU eviction or TTL pruning for in-memory caches
4. Set max size limits (e.g., 100 entries)

**Acceptance:** Exports and notifications persist across restarts. Cache eviction prevents unbounded growth. Tests verify persistence and eviction.
