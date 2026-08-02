# SPEC — count_sessions Fallback Test

> Parent: `2026-08-01-mcp-live-fix` · Auto Bug Loop Iteration 3
> Finding: **Design Compliance F-1 (LOW)** — `review-mcp-live-fix-design-compliance-iter-3.md`

## Problem

The approved preview `preview-count-sessions-fast-path.md` §4 Verification Plan item 1 explicitly requires a Rust test for the **fallback path** (property `rocksdb.estimate-num-keys` unavailable → full scan returns exact count). Implemented tests cover unfiltered parity (`agent_skill_test.rs:273`), empty → 0 (`:308`), and filtered exactness (`:318`) — but NO test forces the property to be unavailable and asserts the full-scan fallback (`rocksdb.rs:730-760`). The fallback *behavior* is correctly implemented — this is a **test-coverage gap vs the design's verification plan**, not a behavior deviation.

## Requirements

### REQ-CFT-001 — Fallback test
A Rust test SHALL force the `rocksdb.estimate-num-keys` CF-property read to be unavailable (or bypass the estimate path) and assert that unfiltered `count_sessions` falls back to the full scan and returns the EXACT count on a seeded store.

### REQ-CFT-002 — Mechanism contained in test
The mechanism to force unavailability SHALL be test-local (e.g., a test-only helper, a code path that makes the property read fail, or mirroring how count_agents/count_skills fallback is tested) — it MUST NOT alter production behavior, add production flags, or require runtime env manipulation.

### REQ-CFT-003 — No regression
The existing fast-path tests (parity, empty → 0, filtered exactness) SHALL remain green and unchanged in intent.

## Non-Goals

- No change to production code unless strictly necessary to make the fallback testable (and even then, minimal and behavior-neutral).
- No change to the estimate fast path.

## Artifacts

- Contract dir: `docs/contracts/2026-08-01-mcp-live-fix/bugs/2026-08-01-count-fallback-test/`
- References: `plan/review/review-mcp-live-fix-design-compliance-iter-3.md` (F-1), `bugs/2026-08-01-count-sessions-fast-path/plan/preview/preview-count-sessions-fast-path.md` §4
