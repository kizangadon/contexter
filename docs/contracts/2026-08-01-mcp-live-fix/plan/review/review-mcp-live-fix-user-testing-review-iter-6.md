# User-Testing Review Report

# 2026-08-01-mcp-live-fix — Auto Bug Loop Iteration 6 (Lean)

> End-to-end repair of the Contexter MCP server (Rust core + Python FastMCP server): all 8 tools + 4 resources return real engine data over live stdio. Iteration 6 delta: bug `2026-08-01-count-memories-invariant-comment` (comment-only update to the `count_memories` estimate fast path in `contexter-core/src/storage/rocksdb.rs`, iter-5 Code Reviewer [LOW] carry-over).

**Verdict:** PASS (class: 33/33 parent ACs + 3/3 bug ACs letter-pass; 0 findings)

2026-08-02 · 33/33 parent AC + 3/3 bug AC passed · User-Testing Validator (iter-6)

---

## 01 · Test Overview

> **Browser & Environment**
> No UI is expected in this contract — code-only design (MCP architecture + data flow, no frontend/wireframe; confirmed by grep: **0 matches** for `UI|wireframe|browser|frontend` in `preview-mcp-live-fix-approved.md`). Environment: branch `feature/mcp-live-fix`, working tree (feature changes uncommitted, original Phase-4 reports immutable). Rust suite: `cd contexter-core && cargo test` = **471 passed, 0 failed** across 24 test binaries (verified at aggregate level). Bridge live E2E: `cd contexter-server && python3 -m pytest tests/core/test_bridge_live_coverage.py -q` = **23 passed in 8.12s** (exercises `count_memories` filtered/estimate paths against the real Rust engine).

> **Test Summary**
> Read the full bug contract (SPEC/ACCEPTANCE/EDGE_CASES), verified: (1) **AC-IV-001** — the invariant caveat comment IS present in `count_memories` fast path (rocksdb.rs lines 1030–1034) with sibling parity; (2) **AC-IV-002** — Rust suite green at **471 passed / 0 failed** (threshold ≥471); (3) **AC-IV-003** — the `git diff` for this bug's delta is comment-only (1 comment line replaced by 4 comment lines; logic/whitespace untouched), confirmed by mtime ordering (rocksdb.rs 18:15:50 is the newest working-tree file; iter-5's docstring file stamped 18:09:08 — no other file changed after this bug's fix); (4) **E2E no-behavioral-effect** — `count_memories` estimate/filter paths re-proven against the real engine via `test_bridge_live_coverage.py` (23 pass, incl. `count_memories({"memory_type": "fact"}) == 2`, exact filtered counts, and post-delete `== 0`); (5) **EC-IV-01…04** — sibling wording matched, adjacent region untouched, companion `memory_index` CF verified to exist (rocksdb.rs lines 129/329/354/375/1649), fmt/clippy unaffected (comment-only, suite compiles clean).

---

## 02 · Results Table

| # | Scope ID | Phase | Status | Evidence |
|---|---|---|---|---|
| AC-IV-001 | invariant caveat comment present | Read | ✅ PASS | rocksdb.rs L1030–1034: "The memory_items CF holds only memory keys — index entries live in the companion memory_index CF — so the estimate is valid ONLY under this invariant; if it breaks, unfiltered counts must not use the estimate." Sibling equivalents verified in `count_sessions` (L742–747: "companion session_index CF … valid ONLY under this variant"), `count_agents` (L1202), `count_skills` (L1384) — all 4 count functions now carry the same caveat. |
| AC-IV-002 | no behavior change — cargo test 471/0 | Test | ✅ | `cargo test` aggregate: **471 tests passed, 0 failed, 0 ignored** (sum over 24 "test result: ok" blocks; grep any "FAILED|failures:" → none). |
| AC-IV-003 | minimal diff (comment-only) | API | ✅ | `git diff -U3 -- rocksdb.rs` for the delta block: `- // for a fast O(1) count instead of a full scan (REQ-S-004).` replaced by `+ // …(REQ-S-004). The` `+ // memory_items CF holds…` `+ // memory_index CF…only under this` `+ // invariant; if it breaks…` — comment lines only; fast path logic below (`if filter.session_id.is_none() …`) byte-identical. mtime newest = rocksdb.rs 18:15:50; prior file (iter-5 delta `test_framework_efs_coverage.py`) 18:09:08. No other working-tree file modified after this bug's change. |
| AC-1 | 8 tools real data over live stdio | Browser | ✅ PASS (re-stated) | Verified iter-1..4 live probes; comment-only delta cannot alter runtime. |
| AC-2 | 4 resources resolve real data | Browser | ✅ PASS (re-stated) | Verified iter-1..4; unaffected. |
| AC-3 | `type` filter on list_skills/search_memories | Browser | ✅ PASS (re-stated) | Verified iter-1..4; unaffected. |
| AC-4 | auth preserved (open + key modes) | Browser | ✅ PASS (re-stated) | Verified iter-1..4; auth probe 105B; unaffected. |
| AC-5 | store_memory persists to engine | Browser | ✅ PASS (re-stated) | Verified iter-1..4 + live bridge roundtrip; unaffected. |
| AC-6 | invalid params structured errors | Browser | ✅ PASS (re-stated) | iter-4: validation-class stderr fsb=0; unaffected. |
| AC-7 | empty datasets graceful | Browser | ✅ PASS (re-stated) | Verified iter-1..4; unaffected. |
| AC-8 | engine failure contained | Browser | ✅ PASS (re-stated) | iter-4: engine 235B, not_found 213B; unaffected. |
| AC-9 | no mocks in live path | Code | ✅ PASS (re-stated) | Verified iter-1..4; comment change in Rust core — no mock surface. |
| AC-10 | suite green; new tests cover repairs | Browser | ✅ PASS (re-stated + re-run) | Rust **471/**0 and bridge live **23 passed** re-run this iteration; Python suite 904/0 from iter-5 re-stated. |
| AC-11 | no stdout pollution | Browser | ✅ PASS (re-stated) | Verified iter-1..4; unaffected. |

---

## 03 · Edge Cases Mapping

| Edge Case | Status | Evidence |
|---|---|---|
| EC-IV-01 — match sibling wording | ✅ PASS | Comment uses the sibling terminology verbatim: "memory_index CF only under this invariant", "companion `*_index` CF", "unfiltered counts must not use the estimate" — mirror of `count_sessions` phrasing, no invented formulation. |
| EC-IV-02 — no adjacent region edits | ✅ PASS | Fast-path code unchanged: filter guard `if filter.session_id.is_none() && …` (L1035–1039), `property_value_cf(self.cf(self.cfs.memory_items)?, "rocksdb.estimate-num-keys")` (L1040–1044), `if let Ok(count) = val.parse::<u64>()` (L1046), and "Fall through to full scan" (L1050) all identical. Comment sits on the same lines the siblings occupy. |
| EC-IV-03 — comment accuracy for memories | ✅ PASS | Companion CF verified real: `memory_index` defined (L129 LZ4/16MB comment), accessed L329/L354/L375, listed in CF map L1649. "memory_items CF holds only memory keys" matches the successes of prefixed entity-key scheme (`KEY_PREFIX_MEMORY`/index CF pattern). No false mechanism claimed. |
| EC-IV-04 — Rust fmt / clippy unaffected | ✅ PASS | Comment-only — no code whitespace change; `cargo test` compiled clean at 471 tests. |
| Parent edge cases (input validation, boundary, error, concurrency, integration — 19 in EDGE_CASES.md, P1 subset incl. empty-engine, invalid params, auth modes, no-mock, stdout-only-frames) | ✅ PASS (re-stated) | All verified iter-1..4 via live probe/bridge; static comment in Rust core cannot impact Python MCP transport behavior. |

---

## 04 · Wireframe / Design Preview Comparison

Design Compliance pre-verified in earlier iterations (including iter-5). Quick visual sanity this iteration: **no UI to render**. The approved design preview (`preview-mcp-live-fix-approved.md`) is a **code-only contract** — Mermaid architecture (client → `run_mcp.py` → FastMCP server → handlers/auth → services → bridge → Rust engine → store), 6-step data flow, frozen API tables (8 tools/4 resources), no wireframe/frontend. Grep confirms 0 matches for `UI|wireframe|browser|frontend`. The count_memories comment edits live in the Rust core storage layer `(contexter-core/src/storage/rocksdb.rs)`, which is *below* the data-flow boundary drawn in the preview — no architecture node, arrow, API shape, or component hierarchy is touched. No layout deviations possible; no comparison report generated.

---

## 05 · Changes from Previous Iterations

| Item | Status |
|---|---|
| Iter-3 MEDIUM (schema-validation stderr 486B) | ✅ Resolved iter-3/4 (fsb=0, engine 235B); verified iter-4 live; not re-broken. |
| Iter-4 [LOW] → iter-5 [LOW] count_memories comment (this bug) | ✅ Resolved this iteration: comment landed (AC-IV-001/002/003 all pass), behavior unchanged (471 Rust + 23 bridge live green). |
| Iter-5 `efs-docstring-truth` | ✅ Closed iter-5 (904 pass, fabricated IDs removed); not re-opened. |
| All iter-1..5 findings | 0 carried into iter-6 entry; all resolved; re-validation confirms no regressions. |

## 06 · Findings Carried Forward

**0** — zero findings this iteration; zero carried forward. (Only item known at iter-6 entry — the count_memories comment LOW — is the bug under test and is now closed.)

---

## 07 · Timing / UX Notes

- No server/browser required for this delta: the changed artifact is a comment block inside a Rust count fast path. Rust compile+test aggregate **471/0** (~seconds from warm target); bridge live E2E **8.12s**; design preview read + grep comparison trivial. Total validation within the 5-minute hard limit.
- Consistency of finding-capture evidence (git diff + mtime + grep) mirrors the discipline used in iter-5 (mtime ordering) and iter-4 (live probes). No contaminate of the frozen Phase-4 baseline: no validator files under `plan/review/` were modified, and no temp files were created (`docs/tests/` untouched, exit 2 = dir absent).

---

## 08 · Verdict

**PASS** — 33/33 parent ACs (re-stated from iter-1..4 live probes + re-run Rust 471/0 and bridge-live 23/23 identity counts re-proven), 3/3 bug ACs for `count-memories-invariant-comment`, all 4 bug EDGE_CASES + parent 19 edge cases mapped, design preview code-only and unaffected, **zero findings of any kind**.

_Generated by User-Testing Validator · 2026-08-02 · Validation Contract: 2026-08-01-mcp-live-fix (iter-6)_