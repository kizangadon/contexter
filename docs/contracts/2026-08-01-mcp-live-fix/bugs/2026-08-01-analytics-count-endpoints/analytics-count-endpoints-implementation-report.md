# Implementation Report — Analytics Count Endpoints

**Bug contract:** `2026-08-01-analytics-count-endpoints`
**Parent:** `2026-08-01-mcp-live-fix`
**Date:** 2026-08-02

## Summary

Added dedicated store-backed `count_agents` / `count_skills` endpoints to the Rust
core (StorageBackend trait → RocksDB → Engine → PyO3 bridge), exposed them through
the Python `StorageEngine` bridge, and switched `AnalyticsService.get_overview` to
use the dedicated counters instead of full-store `list_agents({}, 1_000_000, 0)`
scans. Verified TDD (RED → GREEN) at both the Rust and Python layers.

## Work performed

### Rust core (RED → GREEN)

- **RED:** Added 4 tests to `contexter-core/tests/engine/agent_skill_test.rs`
  (new section "Count agents / skills (REQ-ACE-001)"): `test_count_agents_matches_store`,
  `test_count_agents_with_status_filter`, `test_count_skills_matches_store`,
  `test_count_skills_with_category_filter`. Confirmed 6× E0599 compile failures
  (no `count_agents`/`count_skills` on `Engine`).
- **GREEN:**
  - `src/storage/mod.rs`: trait methods `count_agents(&AgentFilter) -> EngineResult<u64>`
    and `count_skills(&SkillFilter) -> EngineResult<u64>`.
  - `src/storage/rocksdb.rs`: unfiltered counts use the `estimate-num-keys` O(1)
    fast path; filtered counts are exact scans with identical filter semantics to
    `list_agents`/`list_skills` (agent: name lowercase-contains, status equality,
    capability case-insensitive any; skill: name lowercase-contains, category
    `eq_ignore_ascii_case`). `limit`/`offset` ignored.
  - `src/engine/agent.rs` + `src/engine/skill.rs`: Bypass-policy count methods
    ("always reads from L2" — same policy as `count_sessions`/`count_memories`).
  - `src/bridge.rs`: PyO3 `fn count_agents(&self, filter_json: &str) -> PyResult<usize>`
    and `fn count_skills(...)` (catch_panic / from_str / map_err / `as usize`).
- **Wheel rebuilt and installed:** `maturin build --release` produced
  `target/wheels/contexter_core-0.1.0-cp312-abi3-manylinux_2_38_x86_64.whl`;
  `pip3 install --user --break-system-packages --force-reinstall <wheel>` succeeded
  (PEP 668 externally-managed environment — flag required on this system).
- **Live smoke verified:** 3 agents / 2 skills; `count_agents({status:active})` == 3;
  `count_skills({category:dev})` == 1; list parity 3/2.

### Python layer (RED → GREEN)

- **RED:** Updated test files to the count contract (test edits only, no
  implementation): `tests/core/test_bridge.py` (+4 unit tests), `tests/core/test_engine_real.py`
  (`_REAL_ENGINE_METHODS` 36 → 38), `tests/core/test_bridge_live_coverage.py`
  (contract 38, docstring updated, +2 live tests), `tests/core/test_bridge_mock_rejection.py`
  (+2 guard tests), `tests/services/test_analytics_service.py` (get_overview tests
  switched to counts + new AC-ACE-002 spy test), `tests/services/test_analytics_service_live.py`
  (seed 3 agents / 2 skills, parity test EC-ACE-002, health expectation updated),
  `tests/api/test_analytics.py`, `tests/cli/test_status_format.py`,
  `tests/cli/test_cli.py`. Confirmed RED: 16 failed, 7 errors.
- **GREEN:**
  - `src/contexter_server/core/bridge.py`: `async count_agents(filter=None)` /
    `async count_skills(filter=None)` wrappers (camelize keys, default `"{}"`,
    thread-offload via `_run` with the `_SYNC_ENGINE_CLASS` mock guard — EC-ACE-004).
  - `src/contexter_server/services/analytics_service.py`: `get_overview` now gathers
    `count_agents({})` / `count_skills({})` instead of list scans; removed
    `_ANALYTICS_COUNT_SCAN_LIMIT` and the now-dead `_safe_len` helper.
  - Fixed mock-rejection tests: `_StubLikeEngine` needed `count_agents` /
    `count_skills` class attributes declared as MagicMocks so the dispatch guard's
    TypeError path is exercised (previously `getattr` returned `None` → AttributeError).

## Commands executed

| Command | Exit code | Result |
|---|---|---|
| `cargo test --test agent_skill_test` (RED) | 101 | 6× E0599 — expected RED |
| `cargo test --test agent_skill_test` (GREEN) | 0 | 13 passed |
| `cargo test` (workspace, GREEN) | 0 | ~466 tests, 0 failures |
| `maturin build --release` (in contexter-core) | 0 | wheel produced (6.49s incremental) |
| `pip3 install --user --force-reinstall <wheel>` (no flag) | 1 | PEP 668 externally-managed-environment |
| `pip3 install --user --break-system-packages --force-reinstall <wheel>` | 0 | uninstalled old, installed 0.1.0 |
| `python3 -m pytest <targeted suites>` (RED) | 1 | 16 failed, 7 errors — expected RED |
| `python3 -m pytest <targeted suites>` (GREEN) | 0 | 208 passed |
| `python3 -m pytest` (full suite, contexter-server) | 0 | 864 passed, 0 failures |
| `python3 -m ruff check <changed files>` | 0 | All checks passed |

## Acceptance criteria status

| AC | Verdict | Evidence |
|---|---|---|
| AC-ACE-001 (overview counts 3/2 from live seed) | PASS | `test_overview_counts_reflect_seeded_data` (live) |
| AC-ACE-002 (dedicated counts, no list_* scan) | PASS | `test_uses_dedicated_counts_not_full_store_scan` — `count_*` awaited once, `list_*`/`cache_telemetry` not awaited |
| AC-ACE-003 (bridge exposes count_agents/count_skills) | PASS | `_REAL_ENGINE_METHODS` (38), live coverage contract (38), `test_count_agents_live`/`test_count_skills_live` |
| AC-ACE-004 (mock guard covers new methods) | PASS | `test_run_rejects_mock_class_attribute_count_agents`/`..._count_skills` |

Edge cases: EC-ACE-001 (engine error → `_safe_int` degrades to 0 + warning) —
PASS via `test_handles_partial_failure_in_gather` / `test_logs_missing_keys_explicitly`.
EC-ACE-002 (parity count vs list) — PASS via `test_counts_match_list_based_counts`.
EC-ACE-003 (CLI status path renders) — PASS via CLI status tests (864-suite green).
EC-ACE-004 (mock guard) — PASS.

## Notes

- **No commits created.** Working tree remains shared/dirty as instructed; no
  `git reset`, `checkout`, or `stash` was run.
- Unfiltered RocksDB counts use `estimate-num-keys` (O(1), may lag deletes);
  filtered counts are exact scans — documented in `test_bridge_live_coverage.py`.
- Pre-existing cargo warnings (unused test imports, dead `version` field in hnsw.rs)
  are untouched.
- Changed files: `contexter-core/src/storage/mod.rs`, `storage/rocksdb.rs`,
  `engine/agent.rs`, `engine/skill.rs`, `bridge.rs`,
  `contexter-core/tests/engine/agent_skill_test.rs`;
  `contexter-server/src/contexter_server/core/bridge.py`,
  `services/analytics_service.py`; 9 test files under `contexter-server/tests/`.
