# Bug: Missing inline tests across source files

## Problem
28 out of ~33 source files under `contexter-core/src/` lack `#[cfg(test)] mod tests { ... }` blocks per REQ-TST-008. Notably:
- All `engine/` split files (agent.rs, memory.rs, session.rs, skill.rs, settings.rs, maintenance.rs)
- All stub modules (fts, vector, analytics, wal, telemetry, versioning, util, crdt)
- Many model files (notification, feedback, correlation, analytics, skill, telemetry)
- Implementation detail files (storage/types.rs, storage/migrations.rs, cache/mod.rs, cache/metrics.rs, compression/mod.rs, bin/cli.rs)

## Requirements
- REQ-001: Add at minimum a placeholder `#[cfg(test)] mod tests { ... }` with at least one test function to each of the 28 files that are missing them
- REQ-002: For engine/ split files (agent, memory, session, skill), add meaningful unit tests matching what was already in the monolithic engine tests
- REQ-003: Tests MUST compile and pass (`cargo test`)
- REQ-004: Use `#[allow(dead_code)]` in stub module tests if needed
