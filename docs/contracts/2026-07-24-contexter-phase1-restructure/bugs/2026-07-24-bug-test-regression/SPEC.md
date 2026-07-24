# Bug: Unit test count regression

## Problem
Test count dropped from ~194 to 175 — loss of 19 unit tests during restructure:
- `cache/`: 22→13 tests (loss of 9)
- `compression/`: 17→8 tests (loss of 9)
- `types/`→`models/`: 13→11 tests (loss of 2)

These tests were lost during the module split — the content wasn't migrated from old inline tests when files were split.

## Requirements
- REQ-001: Check git history (`git show HEAD~1:contexter-core/src/cache/mod.rs` or the old root src/) to recover lost cache tests and restore them in `cache/dashmap_lru.rs` or appropriate location
- REQ-002: Check git history to recover lost compression tests and restore them in `compression/codecs.rs`
- REQ-003: Check git history to recover lost types/model tests and restore them in appropriate `models/*.rs` files
- REQ-004: Test count MUST be ≥ 194 total after restoration
- REQ-005: `cargo test` must pass with 0 failures
