# Security Review Report

# MCP Server Live-Functionality Repair — Auto Bug Loop Iteration 2

> Security re-review of the ENTIRE feature scope after iter-2 bug-contract fixes (handlers id bounding, handler observability bounds, camelize invariant test, docs corrections incl. SSE/_api_key documentation, max-request-body env canonicalization, launcher exception-type pin) plus NEW code: Rust engine count endpoints (count_agents/count_skills), search total=-1 failure signal, bridge runtime-failure diagnostics logging, launcher RuntimeError pin. Baseline findings F1-F7 and iter-1 findings F-IT1-01..05 re-verified against the current working tree (HEAD 27e031d + uncommitted changes).

**Verdict:** PASS (class: 0 Critical / 0 High / 0 Medium / 0 Low / 0 informational — zero findings of any kind)

2026-08-02 · 0 findings · Security Architect

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 0 |

> **Security Scope**
> Threat-modeled the full changed surface of Iteration 2: (1) re-verification of all 5 iter-1 findings against the current working tree (F-IT1-01 not_found_error id echo, F-IT1-02 handler log bindings, F-IT1-03 resource URI _api_key + SSE gating docs, F-IT1-04 camelize collision invariant, F-IT1-05 CONTEXTER_MAX_REQUEST_BODY canonical); (2) NEW Rust engine count endpoints count_agents/count_skills — filter parsing (typed serde structs, no dynamic query language), filter semantics parity vs list_agents/list_skills at both engine and storage layers; (3) search total=-1 failure signal — sentinel vs silent 0, no information disclosure to clients; (4) bridge runtime-failure diagnostics logging — bounded args summary (64 chars/arg, 200 total), exception TYPE only on stderr, full traceback server-side only, bounded log paths; (5) launcher RuntimeError pin — precise exception type, no bare pytest.raises(Exception) repo-wide; (6) full-scope re-scan: auth constant-time path, canonical env reads, input validation caps, error containment, secrets hygiene, stdout purity, .gitignore/docs-tests hygiene.

---

## 02 · Vulnerability Findings

## No findings — all iter-1 findings resolved, no new findings.

### F-IT1-01 — RESOLVED — not_found_error id echo bounded

**Verified:** `mcp_tools/handlers.py` wraps every not-found id with `_bounded` at all six call sites — L170 (store_memory), L256 (get_session), L319 (get_agent_info), L444 (session_resource), L472 (memory_resource), L500 (agent_resource). `_bounded` (L68-73) caps echoes at 64 chars with a `…` marker. `tests/mcp/test_handlers_id_bounding.py` proves 1 MB id → error message ≤ 256 chars, no raw id echo, 64-char bounded fragment, and byte-identical messages for ≤ 64-char ids (EC-HIB-004). Suite green.

### F-IT1-02 — RESOLVED — handler log bindings bounded

**Verified:** all request-id/project/type log bindings now apply `_bounded`: handlers.py L148 (session_id), L243 (session_id), L277 (project), L306 (agent_id), L334 (type), L431 (session_id), L459 (memory_id), L487 (agent_id). `search_memories` binds no client input at all (L198 bare call_received). `tests/mcp/test_handlers_id_bounding.py` asserts a 1 MB id never appears in joined log output and every binding is ≤ 64 chars; `tests/mcp/test_handler_observability.py` asserts no secrets in logs. Suite green.

### F-IT1-03 — RESOLVED — resource URI `_api_key` + SSE gating documented

**Verified:** README now contains an "MCP Interface (SSE)" section documenting: SSE transport on port 8052, `_api_key` as an optional query parameter on resource URIs (RFC 6570 `{?_api_key}` suffix) for all 4 resources, constant-time key comparison (`hmac.compare_digest`), JSON-RPC error serialization for missing/mismatched keys, and the unset-key development mode. The docs-corrections contract (REQ-DOC-001 / AC-DOC-001) is met. Additionally, FastMCP's SSE transport binds 127.0.0.1 (loopback) by default, mitigating network exposure; the design itself remains the frozen BUG-029 contract (no code change).

### F-IT1-04 — RESOLVED — camelize collision invariant asserted

**Verified:** `tests/core/test_bridge.py` L1005-1079 adds the collision-invariant suite: adversarial pairs (`foo__bar`/`foo_bar`/`fooBar`, `_foo`/`Foo`, `foo_`/`foo`, `a_b`/`ab` trap), documented last-wins policy (REQ-CCI-002), deterministic ordering in both insertion orders, top-level-only camelization, non-string key passthrough. All passing.

### F-IT1-05 — RESOLVED — CONTEXTER_MAX_REQUEST_BODY canonical

**Verified:** `main.py` L206 reads `CONTEXTER_MAX_REQUEST_BODY` (canonical prefix). `tests/api/test_security.py` L198-221 asserts the canonical var drives the body limit AND the legacy bare `MAX_REQUEST_BODY` no longer affects it. Grep of all env reads: every server env var uses the `CONTEXTER_` prefix. Suite green.

### NEW CODE — no findings

- **Engine count endpoints (`count_agents`/`count_skills`):** bridge.rs parses the filter JSON into typed serde structs (`AgentFilter`/`SkillFilter`) via `from_str` — no dynamic query language, no SQL, no string-built queries. Filter predicates (name lowercase-contains, status equality, capability/category case-insensitive equality) are byte-for-byte the same in `count_*` and `list_*` at both the engine layer (engine/agent.rs, engine/skill.rs) and the storage layer (storage/rocksdb.rs). No injection surface in any filter path. Unfiltered counts use `rocksdb.estimate-num-keys` (documented O(1) estimate that can lag deletes; filtered counts are exact scans) — an accuracy nuance, documented in test_bridge_live_coverage.py, not a security issue. Live coverage tests in Rust (agent_skill_test.rs) and Python (test_bridge_live_coverage.py) pass.
- **Search total=-1 signal:** memory_service.py L73-84 — a failed count call is logged explicitly (`search_count_failed`, ERROR) and surfaces `total=-1` to the caller; `-1` is unambiguously distinguishable from a real count (≥ 0) and from a silent 0. Error details stay server-side; the client sees only the numeric sentinel. No information disclosure. `tests/services/test_memory_service.py` L168-199 covers both the sentinel and the happy path.
- **Bridge diagnostics logging:** `_write_runtime_failure_diagnostics` (bridge.py L136-162) persists bounded args summary (≤ 64 chars/arg, ≤ 200 total via `_truncated_args_summary`) + full traceback to the server-side diagnostics log file; stderr receives ONE concise line with exception TYPE only (the structlog `exception` key is deliberately avoided, L244-247) and the diagnostics path is capped at 100 chars (L249-255). The API key never reaches bridge args (stripped at the handler boundary), so no secret can enter the log. Content-prefix residual (64 chars) is the documented, accepted REQ-BH-001..003 cap.
- **Launcher RuntimeError pin:** `tests/mcp/test_mcp_launcher_wiring.py` L218-222 pins `pytest.raises(RuntimeError)` with a comment explaining the empirical engine behavior; grep confirms zero bare `pytest.raises(Exception)` remain repo-wide (REQ-LET-002).

### Prior findings re-verified — no re-statements

Baseline F1 (URI key) — closed by F-IT1-03 resolution. Baseline F2 (input bounds) — closed iter-1 (content 1 MB / query 10k / limit clamps / export allowlist). Baseline F3 (launch traceback) — closed iter-1 (clean stderr + exit 2 + diagnostics log). Baseline F4 (bridge content logging) — closed iter-1 with the accepted 64-char cap (REQ-BH-001..003). Baseline F5 (CONtexTER typo) — closed iter-1 (canonical everywhere; README documents canonical vars only). Baseline F6 (camelize) — closed this iteration (F-IT1-04). Baseline F7 (handler observability) — closed iter-1 (DEBUG per-call logs, INFO lifecycle, ERROR failures, no payloads).

---

## 03 · Security-Critical Code Highlights

- **Constant-time auth intact:** `hmac.compare_digest` in mcp_tools/auth.py L56 and api/deps.py L64; canonical `CONTEXTER_API_KEY` read at auth.py L45, deps.py L51, mcp_server.py L68; byte-identical missing/wrong-key serialization; auth-first in all 8 tools + 4 resources; API key presence-only logging (value never logged).
- **No secrets in new paths:** diagnostics log and stderr line carry bounded args summary and exception type only; grep scan for hardcoded secrets across src/ clean; .gitignore covers `**/docs/tests/`; docs/tests empty (scratch hygiene verified).
- **Count endpoints filter-safe:** typed serde filter structs at the FFI boundary; filter semantics identical to list_*; unknown keys silently dropped by serde and re-applied at the service layer (skill_service defense-in-depth translation) — no filter injection possible.
- **Error containment preserved:** HandlerError/MCPAuthError as isError frames; engine RuntimeError mid-call → structured JSON-RPC error; launch failure → one-line stderr + exit code 2 + full server-side diagnostics; stdout remains pure (JSON-RPC frames only).
- **Input validation unchanged and effective:** content ≤ 1 MB, query ≤ 10k, session limit clamped 0..10_000, search limit 1..100, export allowlist; all bounded error messages (≤ 64-char echoes).
- **Full suite:** 867 passed (including test_handlers_id_bounding, test_handler_observability, test_bridge, test_mcp_auth, test_security, test_mcp_launcher_wiring, test_bridge_engine_failure_stderr, test_error_shape_drift, test_input_validation_gaps, test_mcp_resource_auth_live, test_mcp_type_filter_live, test_memory_service).

---

## 04 · Remediation Recommendations

> **Must Fix**
> None — zero findings across all severity classes (0 Critical / 0 High / 0 Medium / 0 Low / 0 informational).

> **Should Fix**
> None — all iter-1 findings (F-IT1-01..05) verified resolved with regression tests; new code (count endpoints, total=-1, diagnostics logging, launcher pin) introduces no issues.

> **Consider**
> None — no observations, suggestions, or informational notes this iteration.

---

_Generated by Security Architect · 2026-08-02 · Validation Contract: 2026-08-01-mcp-live-fix_
