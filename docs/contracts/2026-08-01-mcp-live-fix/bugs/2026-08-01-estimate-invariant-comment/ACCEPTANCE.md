# ACCEPTANCE — Estimate Fast Path: Document CF Invariant

## AC-EIC-001 — Comment present
- **Given** `contexter-core/src/storage/rocksdb.rs`
- **When** the count_sessions estimate fast path (and count_agents/count_skills paths, if they lacked one) is read
- **Then** a comment states the CF-exclusive-keys invariant and why `estimate-num-keys` is valid only under it

## AC-EIC-002 — No behavior change
- **Given** the full test suites
- **Then** `cargo test` and `python -m pytest -q` remain green (469+ / 881+), with zero logic changes in the diff
