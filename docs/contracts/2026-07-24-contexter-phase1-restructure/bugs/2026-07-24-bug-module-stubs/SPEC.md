# Bug: New module stubs missing sub-module files

## Problem
Several new modules under `contexter-core/src/` exist as stub `mod.rs` files only but the SPEC requires sub-module structure:

- `telemetry/` — needs `metrics.rs`, `reporter.rs` sub-modules
- `crdt/` — needs `merge.rs` sub-module
- `versioning/` — needs `store.rs`, `gc.rs`, `diff.rs` sub-modules  
- `util/` — needs `id.rs`, `time.rs` sub-modules

## Requirements
- REQ-001: Create `telemetry/metrics.rs` with stub MetricsCollector
- REQ-002: Create `telemetry/reporter.rs` with stub MetricsReporter
- REQ-003: Create `crdt/merge.rs` with stub LWW-Merge implementation
- REQ-004: Create `versioning/store.rs`, `versioning/gc.rs`, `versioning/diff.rs` with stubs (diff uses `similar` crate already in deps)
- REQ-005: Create `util/id.rs`, `util/time.rs` with UUID generation and timestamp helpers
- REQ-006: Update each `mod.rs` to declare its sub-modules
- REQ-007: All sub-modules must compile clean with no dead_code warnings (use `#[allow(dead_code)]` where needed)
