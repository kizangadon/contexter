# Bug: Engine module not fully split per SPEC

## Problem
Engine module at `contexter-core/src/engine/` has `mod.rs`, `session.rs`, `memory.rs`, `agent.rs`, `skill.rs`, `settings.rs`, `maintenance.rs` — but SPEC REQ-MOD-008 requires `search.rs`, `export.rs`, `analytics.rs` instead of `settings.rs` and `maintenance.rs`.

Additionally, `engine/mod.rs` is 1,519 lines with ~86% test code — inflating compile time and violating the per-domain split principle.

## Requirements
- REQ-001: Create `engine/search.rs` — move search_memories function from `memory.rs` or `mod.rs` into this file
- REQ-002: Create `engine/export.rs` — export/backup functionality module (stub with content if not implemented)
- REQ-003: Create `engine/analytics.rs` — analytics aggregation module (stub with content if not implemented)
- REQ-004: Keep `settings.rs` and `maintenance.rs` (they're valid additions) but RE-EXPORT them through mod.rs
- REQ-005: Split test code from `engine/mod.rs` into the appropriate per-file `#[cfg(test)]` blocks
