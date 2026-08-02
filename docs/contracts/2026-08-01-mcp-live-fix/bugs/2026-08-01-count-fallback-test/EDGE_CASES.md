# EDGE CASES — count_sessions Fallback Test

## EC-CFT-001 — Forcing unavailability without production flags
Prefer a test-only mechanism: if the property read is a private helper, a unit test may call the scan path directly or inject a failing property source. Avoid adding env vars / config flags to production solely for testing.

## EC-CFT-002 — Empty store fallback
Also assert the fallback on an EMPTY store returns 0 (fallback correctness across both cases).

## EC-CFT-003 — Mixed-store fallback
Seeded store with sessions across multiple projects: fallback returns the TOTAL across all projects (unfiltered), while the filtered path still returns per-project counts.

## EC-CFT-004 — No behavior drift
The fallback test MUST NOT weaken the fast-path tests (e.g., by mocking the property read globally). Keep tests independent.

## EC-CFT-005 — Comment quality
The test SHOULD reference why the fallback exists (EC-CS-002: property unavailable → exact scan) so future readers understand the branch.
