# Bug: Settings Model Missing Analytics Section

**Sources:** SPEC REQ-CFG-003, Design Compliance CFG-1

**File:** `models/settings.py`

**Problem:** The `Settings` model has sections: `project`, `storage`, `cache`, `mcp_server`, `rest`, `llm_providers`, `notifications`, `versioning`, `telemetry`. It is missing the `analytics` section required by REQ-CFG-003.

**Fix:** Add `AnalyticsConfig` Pydantic model with appropriate fields (e.g., `enabled: bool = True`, `retention_days: int = 90`, `track_events: list[str] = ["session", "memory", "search"]`). Add `analytics: AnalyticsConfig` field to `Settings` model. Update `_default_settings()`.

**Acceptance:** `Settings` model has `analytics` section. `get_section("analytics")` returns valid config. Tests pass. `_default_settings()` includes analytics defaults.
