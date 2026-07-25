# Bug: Missing Edge Case Test Coverage

**Sources:** SPEC Compliance edge case gaps, E-014, E-015, E-024, E-025, E-031, E-033

**Files:** `tests/api/test_memories.py`, `tests/api/test_feedback.py`, `tests/core/test_bridge.py`, `tests/services/test_search_service.py`

**Problem:** Multiple edge cases from EDGE_CASES.md lack test coverage:
- E-014: Extremely large request body (>50MB) → 413
- E-015: Concurrent session creation with same ID → one succeeds, one 409
- E-024: Bridge thread pool exhaustion (20 concurrent) → all complete
- E-025: Bridge call timeout → timeout exception
- E-031: Null bytes in search query → 422
- E-033: Very long entity ID (10000 chars) → 422

**Fix:** Add test cases for each uncovered edge case:
- E-014: Test client with large payload → 413
- E-015: asyncio.gather with concurrent creates → 201 + 409
- E-024: Fire 20 concurrent mocked bridge calls → verify all complete
- E-025: Mock slow bridge operation → verify timeout raises
- E-031: Search with null byte → verify 422
- E-033: POST with 10000-char ID → verify 422

**Acceptance:** All 6 edge cases have passing test coverage.
