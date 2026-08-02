# EDGE CASES — Unfiltered `count_sessions` O(1) Fast Path

## EC-CS-001 — Empty sessions CF
`estimate-num-keys` on an empty CF returns 0 — fast path must not misbehave on empty stores (parity test AC-CS-002).

## EC-CS-002 — Property unavailable
If the CF property read fails/returns unavailable, fall back to the exact scan (same `property_value` → fallback pattern as count_agents/count_skills at rocksdb.rs:1156/:1335). Never panic, never return a wrong count silently — mirror the existing fallback exactly.

## EC-CS-003 — Estimate error semantics
`estimate-num-keys` is an estimate per RocksDB docs. Because the sessions CF holds only session keys (index rows live in `session_index` CF), the estimate error is inherent-but-bounded and is a **documented accepted semantics** (same as REQ-ACE-001/REQ-S-004 pattern; `test_bridge_live_coverage.py` docstring). Correctness tests assert parity on freshly seeded stores where the estimate is exact.

## EC-CS-004 — Filtered counts with matching project prefix
Filtered path must remain an exact index-prefix scan — do NOT route filtered counts through the estimate (estimate counts ALL sessions regardless of project).

## EC-CS-005 — Concurrent writes during estimate
RocksDB CF properties are snapshot reads; no special concurrency handling needed beyond what count_agents/count_skills already do (no locking, no transaction).

## EC-CS-006 — No API drift
The Rust method name/signature, the PyO3 export, and the Python bridge wrapper name must stay byte-identical (`count_sessions`) — changing any public name breaks `core/bridge.py`, `analytics_service.py`, CLI status, and existing tests.

## EC-CS-007 — Wheel rebuild
Because this is a Rust change, the extension wheel MUST be rebuilt (`maturin build --release` + reinstall) and the live-engine tests (bridge live coverage) MUST pass against the rebuilt wheel — stale-wheel failures are NOT implementation failures.
