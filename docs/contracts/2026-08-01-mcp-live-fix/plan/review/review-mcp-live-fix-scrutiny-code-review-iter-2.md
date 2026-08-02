# Code Review Report

# MCP Live Fix — Full-Scope Iteration 2 Code Review

> Scrutiny of the entire mcp-live-fix feature scope: 11 iter-1 bug contracts (handlers-id-bounding, pydantic-alias-annotated, max-request-body-env, launcher-exception-type, analytics-count-endpoints, search-total-failure, perf-log-and-bounds-docs, engine-failure-stderr, docs-corrections, parent-edge-case-tests, camelize-invariant-test) plus new code introduced for count endpoints (Rust bridge/engine/storage) and search-total semantics.

**Verdict:** PASS (class: scrutiny-code-review)

2026-08-02 · 60+ (feature branch, uncommitted) files changed · Code Reviewer

---

## 01 · Review Overview

| Metric | Value |
|---|---|
| Files Reviewed | Rust: bridge.rs, engine/agent.rs, engine/skill.rs, storage/rocksdb.rs, storage/column_families.rs, tests/engine/agent_skill_test.rs. Python: core/bridge.py, run_mcp.py, mcp_tools/handlers.py, mcp_tools/errors.py, models/{agent,skill,search}.py, services/{analytics,memory}_service.py, api/deps.py, main.py, cli/status_commands.py, rate_limiter.py. Tests: test_analytics_service.py, test_memory_service.py, test_security.py, test_mcp_launcher_wiring.py, test_handlers_id_bounding.py, test_protocol_edge_cases.py, test_bridge_live_coverage.py, test_bridge.py, test_bridge_engine_failure_stderr.py, test_env_canonicalization.py. |
| Tests Passed | Python 867 + Rust 336 (13 agent_skill_test + 323 lib) |
| Issues Found | 0 |
| Code Coverage | n/a — full-suite evidence used |

> **Scope**
> Iteration 2 full-scope scrutiny. All 11 iter-1 bug contracts verified against implementation and tests; new Rust count endpoints (bridge → engine → RocksDB) reviewed for correctness, filter-guard completeness, semantics parity with list, and FFI error safety; bridge import-guard and camelize helpers reviewed; search-total -1 semantics verified end-to-end; docs requirements (REQ-DOC-001/002/003, REQ-PLB-002) verified in README + design doc.

---

## 02 · Code Diff Review

All changes shown with unified diff. **60+ files** changed.

### full feature-branch diff (HEAD 27e031d, uncommitted)

```diff
Covered by file-by-file verification above (diff is 60+ files; each reviewed against its bug contract and acceptance criteria).
```

Diff data: `git diff --stat HEAD (uncommitted working tree)`

---

## 03 · Review Findings

### Findings: ZERO (0)

No findings of any kind were identified in this iteration. The full verification matrix follows.

---

### Iter-1 bug contracts — all 11 RESOLVED (verified against source + tests)

| Contract | Verification evidence |
|---|---|
| `handlers-id-bounding` | All 6 `not_found_error(...)` sites in `mcp_tools/handlers.py` (L170, 256, 319, 444, 472, 500) wrap ids with `_bounded()`; all 16 `_bounded()` call sites present; per-call log bindings (`session_id`, `project`, `agent_id`, `memory_id`, `type`) are all bounded; validation errors use static strings, no user content interpolation. |
| `pydantic-alias-annotated` | `AliasFieldInfo(FieldInfo)` + `Annotated[...]` alias strategy in `models/agent.py` + `models/skill.py`; `ConfigDict(populate_by_name=...)`; full suite run shows ZERO `UnsupportedFieldAttributeWarning`. |
| `max-request-body-env` | Canonical `CONTEXTER_MAX_REQUEST_BODY` is the only name read in production source; grep confirms zero legacy `MAX_REQUEST_BODY`/`CONtexTER_`/`n*` reads in `src/`; `test_security.py` L207-223 (`test_legacy_env_name_ignored`, `test_invalid_env_value_preserves_parsing_behavior`) pins behavior. |
| `launcher-exception-type` | `test_mcp_launcher_wiring.py` L222 pins `pytest.raises(RuntimeError)` for `test_build_services_still_raises_raw_on_engine_open_failure`; SystemExit pinned at L158/183/202; grep confirms ZERO `pytest.raises(Exception|BaseException)` in the whole test tree. |
| `analytics-count-endpoints` | `test_analytics_service.py` L155-173 (`test_uses_dedicated_counts_not_full_store_scan`) asserts `count_agents`/`count_skills` awaited once AND `list_agents`/`list_skills` `assert_not_awaited` (REQ-ACE-003/004: no full-store scan). |
| `search-total-failure` | `test_memory_service.py` L167-239: count failure surfaces `total == -1` (never silent 0, REQ-STF-001) + ERROR log `search_count_failed`; search failure propagates (EC-STF-001); both-fail propagates (EC-STF-002); truncated page reports full count 42 (EC-STF-004). |
| `perf-log-and-bounds-docs` | All per-call logs (`call_received`, `auth_decision`, `engine_result`) at DEBUG in `handlers.py`; README `### Accepted performance decisions` (L279-305) documents 100-item list cap, sequential `store_memory` calls, 10k/entity export cap + LRU-100 cache. |
| `engine-failure-stderr` | `run_mcp.py`: `ENGINE_OPEN_EXIT_CODE = 2`, `DEFAULT_LAUNCH_LOG = ~/.contexter/logs/mcp-launch.log`, `CONTEXTER_LOG_FILE` override; clean single-line stderr + full diagnostics (structured event + traceback) written to launch log. |
| `docs-corrections` | README L114-138: MCP SSE section documents `_api_key` as tool argument AND `{?_api_key}` query suffix on all 4 resource URIs (REQ-DOC-001); design doc §7.4 telemetry table corrected to snake_case (`entries_by_type`, `total_ops`) with camelCase note (REQ-DOC-002); README L248-256 documents engine lowercases memory content (REQ-DOC-003). |
| `parent-edge-case-tests` | `test_protocol_edge_cases.py` present: `TestWrongJsonRpcPayload` (L204), `TestConcurrentToolCalls` (L242), `TestConcurrentStoreMemorySameSession` (L364), `TestClientDisconnect` (L419) — covers EC-015/017/018/021. |
| `camelize-invariant-test` | `test_bridge.py` L1007-1080: collision policy documented (many-to-one, deterministic, last-wins); adversarial pairs tested — `foo_bar`/`fooBar`, `foo__bar` collapse, `a_b`→`aB` trap, reversed insertion order. |

### New code review (Rust count endpoints, bridge wrappers, search-total)

**Rust `count_agents`/`count_skills` (contexter-core):**
- `bridge.rs` L323/L397: `catch_panic` → `PyValueError` on filter parse, `map_err` on engine error, `usize` return — no silent defaults, no panics crossing FFI.
- `engine/agent.rs` + `engine/skill.rs`: policy "Bypass — always reads from L2" — count must be authoritative, correct choice.
- `storage/rocksdb.rs`: unfiltered count uses `rocksdb.estimate-num-keys` (O(1), mirrors pre-existing `count_memories` L989) with full-scan fallback; filtered counts do a full scan.
- Estimate-path guard covers ALL filter fields (`AgentFilter`: name/status/capability; `SkillFilter`: name/category) — no filter field bypasses the scan.
- Count semantics match list semantics exactly: case-insensitive `contains` for name, `eq_ignore_ascii_case` for capability/category, equality for status — count == len(list) invariant holds.
- Both CFs contain only entity keys (all writes use `agt:`/`skl:` prefixes from `column_families.rs`) — the estimate shortcut is sound.
- `create_memory_bytes`/`update_memory_bytes`: `NewMemory.content` required vs `MemoryPatch.content: Option<String>` — asymmetry is correct (partial update semantics).

**Python bridge (`core/bridge.py`):**
- Import guard replaces bare `contexter_core` import: raises `ImportError` with install instructions; `Mock` imported only for detection/rejection (`test_bridge_exposes_only_real_engine_methods`).
- `_snake_to_camel`/`_camelize_payload_keys` helpers; diagnostics helpers; count wrappers pass through engine errors.
- `test_bridge_live_coverage.py`: `test_count_agents_live`/`test_count_skills_live` exercise the real engine; `test_every_contract_method_is_exercised` guards completeness.

**Search total (REQ-STF):** `models/search.py` `SearchResponse.total: int = 0` accepts `-1` (no validation constraint rejects it — confirmed by passing `test_count_failure_surfaces_negative_total`); `memory_service.search` gathers results+count concurrently, never masks count failure.

### Test evidence

- Python suite: **867 passed, 0 failed**; only 1 warning (unrelated starlette `PendingDeprecationWarning` for `python_multipart`); ZERO `UnsupportedFieldAttributeWarning`.
- Rust: `cargo test --test agent_skill_test` → **13 passed** (incl. `test_count_agents_with_status_filter`); `cargo test --lib` → **323 passed**.
- Installed Rust wheel exposes `count_agents`/`count_skills` (verified via `hasattr` on `Engine`).

### Prior findings re-stated

- Phase-4 baseline (7 findings): all resolved per iter-1 report — no re-occurrence in this iteration.
- Iter-1 (3 low findings): all resolved — none re-occur in this iteration.

---

## 04 · Summary & Recommendations

> **Code Quality Assessment**
> Excellent. The iter-1 findings are fully resolved with precise, minimal fixes: `_bounded()` applied uniformly at every echo site, `AliasFieldInfo` cleanly eliminates the pydantic warning, env canonicalization is complete (zero legacy reads), launcher exceptions are pinned exactly, count endpoints avoid full-store scans with dedicated `count_*` engine methods, and search failure semantics never mask a failed count (total=-1 + ERROR log). New Rust count code follows the established `count_memories` pattern, guards every filter field, and mirrors list semantics exactly — the count == len(list) invariant holds. The bridge import guard prevents mock-silent runs entirely. Documentation (README + design doc) now reflects the implemented behavior.

> **Strengths**
> 1. `_bounded()` applied at all 6 not-found sites and all per-call log bindings — no unbounded id echo remains.
> 2. Count endpoints use dedicated engine methods with O(1) estimate path only when no filter is present, and every filter field is guarded.
> 3. Count/list semantics parity is exact (case-insensitive contains, eq_ignore_ascii_case) — no drift between count and list.
> 4. Search total failure surfaces -1 + ERROR log, never a silent 0.
> 5. Full-suite evidence: 867 Python + 336 Rust tests pass with zero targeted warnings.

> **Recommended Improvements**
> None in scope. (Informational: `docs/tests/iter2/` scratch files `live_e2e.py`/`seed_engine.py` belong to the still-running parallel User-Testing Validator and will be cleaned by that validator before session end.)

---

_Generated by Code Reviewer · 2026-08-02 · Validation Contract: 2026-08-01-mcp-live-fix_
