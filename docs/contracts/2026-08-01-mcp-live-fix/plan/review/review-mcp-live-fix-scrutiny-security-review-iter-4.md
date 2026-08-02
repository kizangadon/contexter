# Security Review Report

# MCP Server Live-Functionality Repair — Auto Bug Loop Iteration 4

> Security re-review of the ENTIRE feature scope after iter-4 bug-contract fixes, with emphasis on the 8 new iteration-4 contracts: (1) 2026-08-01-fastmcp-filter-coverage — framework logging filter extended to the function_prompt emitter, the sampling emitter, and the schema-validation WARNING; (2) 2026-08-01-count-fallback-test — `#[cfg(test)]`-only seam in rocksdb.rs forcing the exact full-scan fallback; (3) 2026-08-01-success-path-log-hygiene — `analytics.missing_key` WARNING->DEBUG and import-time API-key warning -> DEBUG; (4) 2026-08-01-suite-warning-hygiene — scoped filterwarnings entry for the python-multipart/starlette PendingDeprecationWarning. Re-affirmed iter-2 clean state (count estimate fast path, fastmcp logging filter Option A) and iter-3 resolution (F-IT3-01 extensions) against the current working tree (HEAD 27e031d + uncommitted changes).

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
> Threat-modeled the iter-4 changed surface with adversarial focus on the four named contracts: (1) fastmcp-filter-coverage — can any logging suppression mask authentication/authorization failures or sensitive data; can the filter swallow contexter's own auth/event records (false suppression); is `mcp_server.api_key_configured` / `mcp_tool.auth.*` visibility preserved; any secrets/tokens in logs; drop-policy safety at EVERY level (including `e.log_level` sampling paths); (2) count-fallback-test — is the test-only seam compiled out of production (`cfg(test)`), does it introduce env/false global hooks or behavioral change in non-test builds; (3) success-path-log-hygiene — security-relevant warning downgrades cannot silently disappear from the diagnostics signal; auth ENFORCEMENT byte-identical; missing/wrong key behavior still surfaced at an appropriate level; (4) suite-warning-hygiene — narrow filterwarnings scoping, no blanket ignore, no masking of other deprecation warnings. Plus full-scope re-verification: secrets/tokens in code/tests/logs/docs added this iteration, `_api_key` resource path validation, constant-time key comparison, rate-limiter, error-shape (byte-identical client frames), input caps (1 MB content / 10k query / 100 list limit), no new SQL/command/template-injection surface, and both full suites run read-only.

---

## 02 · Vulnerability Findings

## No findings — zero items of any kind (no observations, no nits, no informational notes, no recommendations).

## 01 · fastmcp-filter-coverage (contract 2026-08-01-fastmcp-filter-coverage) — verified clean

### F-IT3-01 — RESOLVED — emitter/prefix coverage complete
**Verified:** `contexter-server/src/contexter_server/fastmcp_logging.py` now covers all five framework error-record families with exactly-matching prefixes `fastmcp/server/server.py:1290` WARNING, verified against the INSTALLED fastmcp 3.4.0 (`fastmcp~=3.4.0` pin in pyproject.toml): `Error calling tool `, `Error calling sampling tool `, `Error reading resource `, `Error rendering prompt `, `Invalid arguments for tool `. `_EMITTER_LOGGERS` now includes `fastmcp.prompts.function_prompt` and `fastmcp.server.sampling.run` (plus `fastmcp`, `fastmcp.server`, `fastmcp.server.server`); `tests/mcp/test_framework_efs_coverage.py::TestEmitterInventoryDrift` AST-scans the installed package for every `logger.exception/warning/log` site whose message starts with a family marker and asserts (a) the logger is resolvable, (b) it is in `_EMITTER_LOGGERS`, (c) its prefix is covered; the reverse pin asserts no dead prefix accumulates. Both directions enforced — the exact defense-in-depth gap F-IT3-01 described is closed with a regression lock.

### No auth/security-signal suppression (primary security check)
**Verified:** the filter can ONLY drop records whose `getMessage()` starts with one of the five framework prefixes; contexter's own records never match — collision scan of `contexter-server/src` shows the five prefix strings exist nowhere outside `fastmcp_logging.py` itself. The filter is installed ONLY on the five `fastmcp.*` logger names; Python applies only the ORIGINATING logger's filters, so no `contexter_server.*` logger (bridge, handlers, auth, mcp_server, analytics) is ever touched. Authentication decisions remain fully visible: `mcp_tool.auth.missing_api_key` and `mcp_tool.auth.invalid_api_key` remain WARNING-level structlog events on `contexter_server.mcp_tools.auth` and `mcp_server.api_key_configured` remains INFO. No contexter auth/event record is filtered; the count_sessions estimate fast path is untouched by the filter. The sampling path `logger.log(e.log_level, ...)` is covered by the drop-at-EVERY-level policy (REQ-FC-005), which the `TestDropPolicyPinned` suite asserts for DEBUG/INFO/WARNING/ERROR — a necessary and sufficient policy given the sampling path can emit at any level; it cannot suppress anything outside the framework's own error-call messages. The filter writes nothing and cannot widen output: `getMessage().startswith(...)` is O(prefix), constant prefixes, no allocation beyond `getMessage()` — no DoS surface. Install is idempotent (`_INSTALLED_ATTR`) and survives `configure_logging` (handlers-only removal), and the module docstring documents the drop-policy vs the ~583B wrapped line (REQ-FC-006).

### Validation-class margin (REQ-FC-003)
**Verified:** `TestLiveValidationClass.test_schema_validation_failure_stderr_clean_and_bounded` drives a real FastMCP client against the real server with `{"id": 123}` and asserts: no `Invalid arguments for tool` string, no `server.py` file:line, no `Traceback`, no box chars, and stderr <=400B (comfortable margin under the 512B budget). `TestEngineFailureNoFalseSuppression` asserts the bridge `bridge_call_failed` record is still EMITTED (caplog) and failure stderr <=512B — suppression cannot mask contexter's own engine-failure signal.

## 02. count-fallback-test (contract 2026-08-01-count-fallback-test) — cfg(test)-only seam, production-identical

**Verified:** `rocksdb.rs` adds `force_session_count_fallback: bool` to the `RocksDbBackend` struct (L42-46) under `#[cfg(test)]`, initialized `false` in `Self::new()` under `#[cfg(test)]`, and consumed ONLY inside `#[cfg(test)] if self.force_session_count_fallback { return Ok(None); }` in the new `estimated_session_count()` helper (L217-239). The seam field, initializer block, and guard block are all compiled OUT of any non-test build; no env var, no global flag, no runtime hook is introduced. `estimated_session_count()` itself is production code but is behavior-identical to the previous inline fast path: `.ok().flatten().and_then(|v| v.parse::<u64>.ok())` exactly as before — `Ok(None)` (property error/unparseable) falls through to the exact full scan. `test_count_sessions_fallback_exact_on_seeded_store` (6 sessions) and `test_count_sessions_fallback_empty_store_returns_zero` set the seam directly on the backend instance and assert exact full-scan counts — REQ-CFT-001..003 met. Existing parity/empty/filtered tests unchanged and green. The count-sessions estimate path additionally now carries the CF-invariant comment (REQ-EIC-001: sessions CF holds only session keys; index entries live in `session_index` CF; unfiltered estimates valid ONLY under that invariant) — contracts count-estimate-docs (README design decisions + architecture spec §7.5) and estimate-invariant-comment are documentation/comment-only (REQ-ED-004/REQ-EIC-002).

## 03. success-path-log-hygiene (contract 2026-08-01-success-path-log-hygiene) — downgrades safe, auth enforcement byte-identical

- `mcp_server.create_mcp_server()` reads canonical `CONTEXTER_API_KEY`; when set it stays `logger.info("mcp_server.api_key_configured")` — INFO, visible at default level (test pins this). When unset the record is now `logger.debug(...)` — `test_api_key_clears module/import with ZERO warning records and `test_api_key_configured_info_when_set` asserts INFO when set → AC-SH-003 met; the signal is NOT lost (test asserts the DEBUG record exists).
- `analytics.missing_key` is emitted via `logger.debug("analytics.missing_key", ...)` in `_safe_get` (analytics_service.py) with the key name and payload keys — DEBUG signal preserved; `test_services/test_analytics_service.py::test_logs_missing_keys_explicitly` asserts the event is still logged (signal not lost) with the same event name; per-call at DEBUG matches established per-call event policy (PF-05). This is an auth-decision trace, not an auth-enforcement change.
- **Auth enforcement unchanged:** `require_api_key` in `mcp_tools/auth.py` still rejects with `MCPAuthError` (`API key required...` / `Invalid API key.`) using `hmac.compare_digest` constant-time compare (L56); the env read is canonical only. Client-visible `isError` frames are byte-identical — confirmed by the auth matrix suites (test_mcp_auth.py constant-time + missing/wrong rejection tests), all 8 tools, 4 resources `_gui` gated (test_mcp_server.py `test_*_rejects_missing_key` ×12). `channels/analytics_overview` uses `{?_api_key}` template like all other resources.
- No secrets reach any new log line: the downgraded records contain no key material; the wrong/missing-key events log only the event names (no key value). Grep of the analytics service diff shows the DEBUG bindings are `key`/`payload_keys`/`default`/`value_type` — all non-secret.

## 04. suite-warning-hygiene (contract 2026-08-01-suite-warning-hygiene) — narrow, deliberate

- `pyproject.toml` `[tool.pytest.ini_options] filterwarnings` entry: `'ignore:Please use `import python_multipart` instead.:PendingDeprecationWarning:starlette\.formparsers'` — scoped by message regexp AND category AND module regexp; documented with a justification comment (starlette 0.38.6 cap under fastapi 0.115.x, python-multipart 0.0.32 legacy `multipart` import, source fix deferred because starlette >= 0.45 breaks fastmcp transport tests). This cannot mask any OTHER warning (REQ-SW-002/003): any deprecation beyond this exact message/category/module still surfaces. Verified empirically: `python -m pytest -q` reports 904 passed, ZERO warnings — REQ-SW-001 met.

## 04. Re-affirmed iter-2/iter-3 Security State (full-scope re-verification against current tree)

- **Count-`estimate-num-` fast path (iter-2/iter-3):** `count_sessions`, `count_agents`, `count_skills`, `count_memories` remain via `rocksdb.estimate-num-keys` on clean fresh stores; errors/`None`/unparse fall to exact scans; no panic/unwrap/expect; hard-coded constant property string — no injection; estimate-vs-exact divergence after updates/deletes documented in README Design Decisions and architecture spec §7.5 (contract count-estimate-docs, REQ-ED-003 concrete numbers included) — informational analytics, not an authorization input; accepted semantics.
- **FastMCP filter Option A:** unchanged mechanics — filter only drops, never emits; scoped to `fastmcp.*`; idempotent; thread-safe; survives `configure_logging`.
- **Auth:** constant-time `hmac.compare_digest` at both ends (auth.py and api/deps.py); canonical `CONTEXTER_*` env reads only — zero legacy `CONtexTER`/typo references anywhere in `contexter-server/src` or `run_mcp.py` (full grep).
- **Input caps intact:** `errors.py` moves shared bounds (`MAX_CONTENT_LENGTH=1_000_000`, `MAX_QUERY_LENGTH=10_000`, `MAX_LIST_LIMIT=100`, `MAX_SEARCH_LIMIT=100`, `DEFAULT_SEARCH_LIMIT=20`, export allowlist) into the one module handlers import — values unchanged, no new surface.
- **Secrets hygiene:** grep of ALL new/changed code, tests, docs (docs+contracts) in this iteration: no keys/tokens; the only `sk-...` string in the tree is a synthetic `sk-live-` + 100×`a` fixture inside `test_bridge.py::test_secret_like_value_never_appears` which PROVES secret-like values never appear verbatim in bridge logs.
- **Rate-limiter / not found / handler bounds:** `rate_limiter.py` diff is environment read canonicalization only (`CONTEXTER_RATE_LIMIT_ENABLED` / `CONTEXTER_RATE_LIMIT`); handler `_bounded` echo caps (64 chars) and per-call logging unchanged. 
- **Injection surfaces:** no new dynamic SQL/command/template evaluation introduced anywhere in the iteration-4 changes; no CORS/scanners changes this iteration (boundary previously assessed).

## 05. Suite status (read-only)

- Python: `cd contexter-server && python3 -m pytest -q` → **904 passed, 0 failed, 0 warnings** (matches expected 904).
- Rust: `cd contexter-core && cargo test` → **471 passed (24 suites), 0 failed** (matches expected 471).
- No regression observed; new tests for the 8 iteration-4 contracts included in the pass totals.

---

## 03 · Security-Critical Code Highlights

- **Constant-time auth intact:** `hmac.compare_digest` at `mcp_tools/auth.py` require_api_key (missing/wrong key both raise `MCPAuthError`); canonical `CONTEXTER_API_KEY` env read everywhere; `_api_key` value never logged (presence-only signal logged in mcp_server / auth).
- **Auth-decision visibility preserved:** `mcp_server.api_key_configured` INFO (default visible), `mcp_tool.auth.missing_api_key` / `mcp_tool.auth.invalid_api_key` WARNING (default visible) — none reachable by the new filter; launch-time unset-key status present at DEBUG.
- **Filter drops cannot leak:** filter only returns False on 5 constant prefixes; never re-emits; no `getMessage` beyond prefix compare; ZERO framework box/file:line content on failure stderr asserted live (validation class <=400B, error class <=512B), full tracebacks retained server-side in the diagnostics log.
- **Drift regression lock:** `TestEmitterInventoryDrift` AST-scans the INSTALLED fastmcp package and fails if a new emitter logger or error prefix appears uncovered — the security invariant (bounded failure stderr, REQ-FL-001) is now pinned to the actual dependency, with `fastmcp~=3.4.0` pinned in `pyproject.toml`.
- **Test-only seam invisible to production:** `#[cfg(test)]` on all three seam artifacts; production `estimated_session_count()` byte-for-byte equivalent to the previous inline fast path.
- **Zero warnings / zero failures:** `filterwarnings` narrow-scoped; suites at 904 (py) / 471 (rs), no deprecation noise masked.

---

## 04 · Remediation Recommendations

> **Must Fix**
> None — 0 Critical / 0 High / 0 Medium / 0 Low / 0 informational (zero findings of any kind).

> **Should Fix**
> None — all iter-3 findings (F-IT3-01 and the code/user-testing/SPEC-compliance items consolidated into the eight iteration-4 contracts) verified resolved with regression tests.

> **Consider**
> None — no observations, suggestions, or informational notes this iteration.

---

_Generated by Security Architect · 2026-08-02 · Validation Contract: 2026-08-01-mcp-live-fix_
