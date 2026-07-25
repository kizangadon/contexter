# Bug Fix Report: Boost Conformance

**Bug:** Agent/skill name field boost 1.5 → 2.0

## Changes Made

### `contexter-core/src/fts/schema.rs`
- `agent_schema()` line 106: `(name_field, 1.5)` → `(name_field, 2.0)`
- `skill_schema()` line 140: `(name_field, 1.5)` → `(name_field, 2.0)`

### `contexter-core/src/fts/tantivy.rs`
- Line 415: Comment update `1.5×` → `2.0×`
- Line 442: Comment update `boost 1.5 vs 1.0` → `boost 2.0 vs 1.0`

## Verification
- `cargo build --workspace` — passes
- `cargo test --workspace` — all 323 lib tests + all integration tests pass
- `test_field_boosting` passes (verifies name-match ranks higher than content-match)

## Status
✅ FIXED
