# ACCEPTANCE — Suite Warning Hygiene

## AC-SW-001 — Zero warnings
- **Given** `cd contexter-server && python -m pytest -q`
- **When** the suite completes
- **Then** output shows 0 warnings, 881+ passed, 0 failures

## AC-SW-002 — Scoped filterwarnings (if option a)
- **Given** `pyproject.toml`
- **When** the filterwarnings entry is inspected
- **Then** it matches ONLY the specific python-multipart/starlette `PendingDeprecationWarning` (module/type-scoped), carries a justification comment, and does not blanket-suppress

## AC-SW-003 — Other warnings surface
- **Given** the configured warning policy
- **When** a hypothetical unrelated warning occurs
- **Then** it still appears (no global ignore) — verify by checking the filterwarnings scope is narrow

## AC-SW-004 — Suite green
- **Given** the full suite
- **Then** 881 + tests pass, 0 failures (test content unchanged)
