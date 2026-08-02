# User-Testing Review Report

# MCP Live-Fix — Iteration 4 (Filter Coverage Completion + Docs/Test Precision Contracts)

> Auto Bug Loop iter-4 re-validation of 8 bug contracts: AC-FC-001..007 (fastmcp filter coverage — schema-validation emitter now fully suppressed), AC-ED-001..004 (estimate semantics docs), AC-EIC-001..002 (CF-invariant comment), AC-CFT-001..004 (count fallback test), AC-EP-001..004 (EFS test precision), AC-SL-001..003 (session test limit pin), AC-SH-001..005 (success-path log hygiene), AC-SW-001..004 (suite warning hygiene) against the live stdio MCP server and full Rust/Python suites.

**Verdict:** PASS (class: 33/33 bug-ACs letter-pass; 0 findings carried forward)

2026-08-02 · 33/33 AC passed · User-Testing Validator (iter-4)

---

## 01 · Test Overview

> **Browser & Environment**
> App started fresh per scenario: `python3 contexter-server/run_mcp.py` via Popen (stdio pipes, CONTEXTER_PATH/CONTEXTER_API_KEY set, ANSI-stripped stderr drain-thread, readiness poll on `Starting MCP server`), engine dirs freshly seeded per run (empty, 12-session, corrupt dir), Python 3.12.3, FastMCP 3.4.0, wheel contexter_core rebuilt. Launch-probe variants rc=0 / rc=2. Suite: `python -m pytest -q` 904 passed / 0 warnings (26.13s), `cargo test` 469 passed 0 failed. Scratch under /tmp/opencode (kept) + docs/tests (deleted).

> **Test Summary**
> Iter-4 aimed to CLOSE the iter-3 medium finding (schema-validation WARNING class survived the fastmcp filter: 486B failure stderr with file:line ref). New evidence: validation-class failure (`get_session` with int id 123, `store_memory` empty content) now emits ZERO bytes to failure stderr (fsb=0, no box, no traceback, no file:line) — the schema-validation emitter is now fully covered (AC-FC-002). Regression matrix re-measured live: engine 235B (=195 + log_path len, formula-exact), not_found 213B, auth-missing 105B, auth-wrong 105B — all failure_specific_bytes non-negative and <=512. Success path (open + key mode): ZERO WARNING-level records when only success calls run (INFO lifecycle only); the ONLY warning seen in a mixed stream was a single `mcp_tool.auth.missing_api_key` atom from a deliberately key-less resource call (auth-reject path — contract-preserved, AC-SH-004). Corrupt-engine launch: rc=2, one line, no preamble. Clean open-mode launch: rc=0, INFO startup only. Docs contracts satisfied in README + arch spec (concrete measured numbers 100/100→100/100, +100 updates→200, +50 deletes→150, flush→210) and rocksdb.rs comment; Rust count_fallback + concurrent-pin tests pass. Full suites green: 904 Python + 469 Rust.

---

## 02 · Acceptance Criteria Results

| AC | Phase | Status | Evidence |
|---|---|---|---|
| AC-FC-001 · all emitters covered | Code+Suite | ✅ PASS | `_EMITTER_LOGGERS` enumerates fastmcp.server.server/.prompts.function_prompt/.server.sampling.run; prefix set covers `Invalid arguments for tool `; drift test `test_emitter_inventory_fully_covered` fail-if-uncovered, green |
| AC-FC-002 · schema-validation ≤400B, no box/traceback/file:line | Live stdio | ✅ PASS | live: invalid args calls (`get_session {"id":123}`, store_memory empty) → fsb=**0B**, 0 box chars, 0 traceback, 0 `file:line` (iter-3: 486B + file:line; iter-3 medium finding RESOLVED) |
| AC-FC-003 · prompt & sampling emitters (unit) | Suite | ✅ PASS | test_framework_efs_coverage.py `test_sampling_emitter_dropped`, `test_prompt_emitter_dropped` etc. — dropped from TRUE originating loggers: 59/59 iter-4 unit tests green |
| AC-FC-004 · no false suppression | Live stdio | ✅ PASS | bridge ERROR `bridge_call_failed` + `handler_error` still emitted; success-path stream INFO-only (keyok run: 0 warn lines on 3 success tool+resource calls) |
| AC-FC-005 · drift test present & green | Unit | ✅ PASS | test_framework_efs_coverage.py::TestEmitterInventoryDrift enumerates installed fastmcp sites + asserts 3 documented emitters present; passes |
| AC-FC-006 · drop-policy documented | Code-read | ✅ PASS | fastmcp_logging.py docstring documents drop-not-downgrade rationale + why dropping is required; test asserts policy holds |
| AC-FC-007 · suite green | Suite | ✅ PASS | 904 passed / 0 failures / 0 warnings |
| AC-ED-001 · README caveat present | Code-read | ✅ PASS | README Design Decisions (lines 298-328): estimate-num-keys exact-on-fresh, inflated after updates/deletes until compaction, flush() does NOT correct, exactness via filtered counts / list tools |
| AC-ED-002 · arch-spec caveat present | Code-read | ✅ PASS | docs/design/specs/2026-07-23-contexter-system-architecture.md count-endpoints section: same caveat, consistent |
| AC-ED-003 · concrete measured numbers | Code-read | ✅ PASS | 100 creates→100; +100 updates→**200 vs 100**; +50 deletes→**150 vs 50**; after flush()→**210 vs 100** |
| AC-ED-004 · docs-only, suite green | Code-read + Suite | ✅ PASS | caveat lives only in README.md + arch spec (attributed docs); no estimate logic churn in this contract; 904/469 green |
| AC-EIC-001 · CF-invariant comment | Code-read | ✅ PASS | contexter-core/src/storage/rocksdb.rs (line ~742): comment states CF-exclusive-keys invariant + why estimate-num-keys valid only under it |
| AC-EIC-002 · no behavior change | Suite | ✅ PASS | cargo 469 passed, pytest 904 passed; diff for this contract is comment-only |
| AC-CFT-001 · fallback test exists & exercises property-unavailable branch | Rust suite | ✅ PASS | `cargo test --lib count_sessions_fallback` → 2 passed (property-unavailable branch + seeded-store exactness) |
| AC-CFT-002 · exact count on fallback | Rust suite | ✅ PASS | seeded store → fallback returns exactly N (full-scan correctness) |
| AC-CFT-003 · fast-path tests unaffected | Rust suite | ✅ PASS | full cargo 469 passed 0 failed |
| AC-CFT-004 · suite green | Suite | ✅ PASS | cargo 469 + pytest 904 green |
| AC-EP-001 · no redundant assertion | Code-read | ✅ PASS | test_engine_fs (test_bridge_live_coverage.py) contains NO `len(stderr)<=n*LIMIT`; failure_specific_bytes computed as monotonic delta (non-negative by construction) |
| AC-EP-002 · docstring accurate | Code-read | ✅ PASS | module docstring describes in-process capfd model; explicitly notes bridge line covered live E2E |
| AC-EP-003 · evidence computation non-negative | Live stdio + Unit | ✅ PASS | live failure_specific_bytes: engine=0 (235 total engine bytes =195+40 log path), not_found=0, auth=0, validation=0 — all non-negative and <=512 |
| AC-EP-004 · suite green, 13 EFS tests discriminating | Suite | ✅ PASS | EFS module 13 tests green; full 904 green |
| AC-SL-001 · explicit limit in filter | Code-read + Rust | ✅ PASS | session_test.rs filter build_limit(u64::uMAX) explicitly (not SessionFilter::default()); `test_concurrent_operations` 1 passed |
| AC-SL-002 · test intent intact (100 writes present) | Rust suite | ✅ PASS | test co-drive 4 threads × 25 ops = 100; asserts all visible with explicit limit |
| AC-SL-003 · suite green | Suite | ✅ PASS | cargo 469 / pytest 904 green |
| AC-SH-001 · no WARNINGs on success path | Live stdio | ✅ PASS | open+key mode, 5 success tool/resource calls (get_system_health, store_memory, analytics overview _with_ key, overview _without_ key) → **0 warning lines**; INFO-only (also re-measured pure stream) |
| AC-SH-002 · signal preserved at DEBUG | Unit | ✅ PASS | test_analytics_service.py::test_logs_missing_keys_explicitly asserts analytics.missing_key logs at DEBUG (not WARNING) and remains visible (signal not lost) — 27 tests green in that module (DEBUG observe path contract-attested rather than live-visible; level pin unit-proof) |
| AC-SH-003 · clean launch stderr | Live launch | ✅ PASS | open mode: rc=0, INFO-only startup, no preamble; corrupt engine: rc=2, exactly ONE line (`contexter: engine_open_failed: ... full diagnostics: <log>`), no box/traceback, 0 stdout bytes |
| AC-SH-004 · auth enforcement unchanged | Live stdio | ✅ PASS | missing/wrong key client frames byte-identical: `Error calling tool 'get_session': API key required. ...` / `Invalid API key.`; single reject WARNING remains (contract-preserved); AC-matrix tests green |
| AC-SH-005 · suite green | Suite | ✅ PASS | 904 passed / 0 failed / 0 warnings |
| AC-SW-001 · zero warnings | Suite | ✅ PASS | `python -m pytest -q` full → 0 warnings summary line; 904 passed |
| AC-SW-002 · scoped filterwarnings only (option a) | Code-read | ✅ PASS | pyproject.toml filterwarnings matches ONLY `python-multipart` starlette PendingDeprecationWarning (module/type-scoped) + justification comment; no blanket ignore |
| AC-SW-003 · other warnings surface | Code-read | ✅ PASS | scope is the single module/type; an unrelated warning would NOT be filtered (narrow scope confirmed) |
| AC-SW-004 · suite green | Suite | ✅ PASS | 904 / 0 / 0 |

---

## 03 · As-Built End-to-End Data Flow

**Interaction:** MCP client (stdio) → run_mcp.py launcher → create_mcp_server (FastMCP 3.4.0) → bridge.py (StorageEngine via ThreadPoolExecutor) → contexter_core RocksDB. Failure paths → handler `Error calling tool ...` isError frames; fastmcp_logging.py Filter (fastmcp namespace) drops framework error/schema-validation records (fsb=0); full diagnostics still written to CONTEXTER_LOG_FILE; unfiltered counts use rocksdb estimate-num-keys, fallback=full scan.

### Request Track · Forward Flow (1→5)

| Step | Layer | Action |
|---|---|---|
| 1 | User | MCP client issues JSON-RPC initialize/initialized, then tool/resource requests |
| 2 | Frontend | stdio protocol routed to launcher preamble and ready marker |
| 3 | API | FastMCP 3.4.0 dispatch: tool/resource handler + per-handler require_api_key |
| 4 | Service | handlers.py domain calls services (session/memory/analytics) → bridge.py StorageEngine |
| 5 | Database | contexter_core RocksDB read/write; unfiltered count → estimate-num-keys / fallback scan |

**Layer Details (Request):**

> **User Layer:** Client JSON-RPC envelope; auth key passed as `_api_key` (tool args or URI query)
>
> **Frontend Layer:** launcher with readiness marker, stderr drainage, CONTEXTER_PATH handling
>
> **API Layer:** FastMCP 3.4.0 + auth gate; filter on fastmcp namespace
>
> **Service Layer:** handler + analytics service; DEBUG-visible missing-key events
>
> **Database Layer:** RocksDB estimate/fast-scan; CF invariant comment

### Response Track · Return Flow (6→10)

| Step | Layer | Action |
|---|---|---|
| 6 | Database | RocksDB ticks results; estimate/fallback count exactness verified in Rust |
| 7 | Service | bridge returns structured error / payload |
| 8 | API | handler maps to `Error calling tool 'X': ...` frames; fsb filtered |
| 9 | Frontend | error frames serialized to client over stdio |
| 10 | User | Client sees identical auth/error frames; success-path frames JSON-RPC only; stderr INFO+bounded |

**Layer Details (Response):**

> **Database Layer:** estimate-returned counts correct (fresh exact; inflated after mutation as documented)
>
> **Service Layer:** bridge failure line only on failures
>
> **API Layer:** client frames unchanged; validation fsb=0
>
> **Frontend Layer:** pure JSON-RPC stream; debug signal at DEBUG
>
> **User Layer:** client receives exact frames; auth reject leaves single WARNING (preserved)

**Trace (Response):** DB: estimate/fallback → count or error → Service: bridge → mapped error → API: filter suppresses; frame passes → Frontend: stdio serialization

**33/33** AC passed

---

## 04 · Test Steps Executed

1) 8 contracts read; 2) full suites (904 pytest, 469 cargo); 3) live stdio harness: 9 error classes + both auth modes + success-path + launches (ready-on marker); 4) schema-validation regression probe (iter-3 finding); 5) key-only success stream (0 warnings); 6) fail-verif: key key possible frame + WARNING attribution; 7) DEBUG-level unit pins; 8) docs/rocksdb/CF-invariant code-read; 9) docs/tests deleted; report generated.

---

## 05 · Expected vs Actual

| | Description |
|---|---|
| **Expected** | iter-3 finding (schema-validation 486B with file:line) resolved; all 8 iter-4 bug contracts letter-pass; success-path WARN-free in both modes; suite cargo+pytest green; docs accurate. |
| **Actual** | fsb=0 for schema-validation failures; validation-matrix zero bytes; success streams 0 warnings; single rejection-WARNING attributed to deliberate keyless auth call (preserved; AC-SH-004); 904+469 suites green, 0 warnings; docs (README/specs) + rocksdb comment satisfy doc contracts; iter-3 medium finding CLOSED. |

No wireframe/visual UI in scope — this is a stdio MCP server; per-phase equivalence compared against iter-3 baselines (bytes before/after). All 8 contracts verify.

---

_Generated by User-Testing Validator (iter-4) · 2026-08-02 · Validation Contract: 2026-08-01-mcp-live-fix_
