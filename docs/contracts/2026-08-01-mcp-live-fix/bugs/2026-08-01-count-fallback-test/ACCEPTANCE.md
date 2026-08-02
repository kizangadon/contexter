# ACCEPTANCE — count_sessions Fallback Test

## AC-CFT-001 — Fallback test exists and passes
- **Given** the Rust test suite (`cargo test` in `contexter-core/`)
- **When** the fallback test runs
- **Then** it passes, and it specifically exercises the property-unavailable branch (not just the fast path)

## AC-CFT-002 — Exact count on fallback
- **Given** a seeded store (e.g., N sessions across projects)
- **When** the fallback path is forced and unfiltered `count_sessions({})` runs
- **Then** it returns exactly N (full scan correctness)

## AC-CFT-003 — Fast-path tests unaffected
- **Given** the existing count_sessions tests (parity, empty → 0, filtered exactness)
- **When** the suite runs
- **Then** all remain green; the fallback test adds coverage rather than replacing

## AC-CFT-004 — Suite green
- **Given** the full Rust suite
- **Then** `cargo test` shows 469 + new tests passed, 0 failed; full Python suite stays 881 passed / 0 failures
