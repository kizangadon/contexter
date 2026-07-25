# Bug: Unvalidated `data: dict` Endpoints

**Sources:** Security HI-01, Code Reviewer P3 #11

**Files:** `api/feedback.py`, `api/settings.py`, `api/onboarding.py`, `api/files.py`

**Problem:** 5 endpoints accept bare `data: dict` with zero Pydantic validation. Any arbitrary JSON payload is accepted. The settings endpoint writes user-controlled data to config.yaml.

**Fix:**
1. `api/feedback.py`: Create `BugReport` and `FeatureSuggestion` Pydantic models
2. `api/settings.py`: Create `SectionUpdate` model or use specific per-section models
3. `api/onboarding.py`: Create `WizardData` Pydantic model
4. `api/files.py`: Create `WatchFilesRequest` Pydantic model

**Acceptance:** All previously `data: dict` endpoints now accept typed Pydantic models with validation tests.
