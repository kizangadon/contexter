# Security Review Report

# Contexter Phase 3 Python API Layer — Iteration 3 (Final Re-Verification)

> Auto Bug Loop Iteration 3 — confirms all 3 Iteration 2 findings (BUG-028, BUG-029, BUG-030) are resolved. Full-surface scan for new vulnerabilities.

**Verdict:** PASS (class: green)

2026-07-26 · 0 new findings — all 3 remediations verified · Security Architect

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 0 |
| Informational | 0 |

> **Security Scope**
> Complete re-verification of all 3 Iteration 2 findings: NEW-01 (BUG-028 — MCP auth timing-safe comparison), NEW-02 (BUG-029 — MCP resource authentication bypass), NEW-03 (BUG-030 — path confinement). Full-surface discovery scan across FastAPI REST layer, MCP tools/resources layer, settings service, rate limiting, body size enforcement, dependency security, and logging.

---

## 02 · Vulnerability Findings

### All 3 Iteration 2 Findings — RESOLVED

No new vulnerabilities discovered. Full test suite: **608/608 passed**.

---

#### ✅ BUG-028 (LOW — Iteration 2 NEW-01) — MCP auth timing-safe comparison — RESOLVED

**Finding:** `mcp_tools/auth.py` used `!=` for API key comparison while REST layer (`deps.py`) used `hmac.compare_digest()`.

**Fix verification:**
- **Source:** `/home/don/Code/contexter/contexter-server/src/contexter_server/mcp_tools/auth.py`, line 56
- **Implementation:**
  ```python
  if not hmac.compare_digest(api_key, expected):
  ```
- **`import hmac`** present at line 7 ✅
- **Consistency:** Both MCP auth (`auth.py` line 56) and REST auth (`deps.py` line 64) now use `hmac.compare_digest()` with identical functional signature ✅
- **Test coverage:** `tests/mcp/test_mcp_auth.py` — 8 tests covering no-key-configured, valid, missing, None, empty, wrong key, and ValueError subtype. All pass.
- **Verdict:** ✅ **Fully resolved.** Timing-side-channel defense is now consistent across both REST and MCP authentication layers.

---

#### ✅ BUG-029 (MEDIUM — Iteration 2 NEW-02) — MCP resource authentication bypass — RESOLVED

**Finding:** 4 MCP read-only resource handlers (`session_resource`, `memory_resource`, `agent_resource`, `analytics_overview_resource`) had no `_api_key` parameter and did not call `require_api_key()`.

**Fix verification:**

**Handler layer** (`mcp_tools/handlers.py`):

| Resource Handler | `_api_key` param | `require_api_key()` call |
|---|---|---|
| `handle_session_resource` (line 225) | `_api_key: str \| None = None` ✅ | Line 229: `require_api_key(_api_key)` ✅ |
| `handle_memory_resource` (line 244) | `_api_key: str \| None = None` ✅ | Line 247: `require_api_key(_api_key)` ✅ |
| `handle_agent_resource` (line 262) | `_api_key: str \| None = None` ✅ | Line 265: `require_api_key(_api_key)` ✅ |
| `handle_analytics_overview_resource` (line 278) | `_api_key: str \| None = None` ✅ | Line 282: `require_api_key(_api_key)` ✅ |

**Registration layer** (`mcp_server.py`):

| Resource Registration | `_api_key` param passes through |
|---|---|
| `session_resource` (line 207) | `_api_key=_api_key` at line 212 ✅ |
| `memory_resource` (line 219) | `_api_key=_api_key` at line 224 ✅ |
| `agent_resource` (line 231) | `_api_key=_api_key` at line 236 ✅ |
| `analytics_overview_resource` (line 240) | URI `{?_api_key}` — `_api_key=_api_key` at line 246 ✅ |

- **Test coverage:** `TestResourceAuth` class in `tests/mcp/test_mcp_server.py` — 12 tests (3 per resource: missing key, wrong key, valid key). All pass.
- **Verdict:** ✅ **Fully resolved.** All 4 MCP read-only resources are now authenticated with the same `require_api_key()` mechanism as the 8 MCP tools. The URI pattern `{?_api_key}` for `analytics_overview_resource` correctly supports the optional query-parameter pattern for resources.

---

#### ✅ BUG-030 (INFO — Iteration 2 NEW-03) — Path confinement — RESOLVED

**Finding:** `validate_safe_path()` checked for `..` traversal but did not restrict paths to an allowed base directory.

**Fix verification:**
- **Source:** `/home/don/Code/contexter/contexter-server/src/contexter_server/api/files.py`
- **Signature updated** (line 27): `def validate_safe_path(path: str, base_dir: str | None = None) -> Path:`
- **Base-directory confinement** (lines 72–78):
  ```python
  if base_dir is not None:
      resolved_base = os.path.abspath(base_dir)
      if os.path.commonpath([str(abs_path), resolved_base]) != resolved_base:
          raise HTTPException(status_code=403, detail="Path outside allowed directory")
  ```
- **`list_files` endpoint** (line 95): `validate_safe_path(path, base_dir=os.getcwd())` ✅
- **Docs updated:** Docstring explains `base_dir`, `Returns`, and `Raises` (400 vs 403 distinction) ✅
- **Test coverage:** `tests/api/test_security.py` — 6 specific confinement tests:
  - `test_path_outside_base_dir_rejected` (integration)
  - `test_path_within_base_dir_accepted` (unit)
  - `test_path_exactly_base_dir_accepted` (edge case)
  - `test_path_outside_base_dir_rejected` (unit)
  - `test_base_dir_prefix_not_confused` (boundary — `/tmp2` vs `/tmp`)
  - `test_base_dir_none_skips_confinement` (backward compat)
- **Verdict:** ✅ **Fully resolved.** The function now enforces both `..` traversal prevention AND base-directory confinement with proper 400/403 distinction. Backward compatible when `base_dir=None`.

---

## 03 · Security-Critical Code Highlights

### Authentication — consistent timing-safe comparison across all layers ✅

**REST API** (`deps.py` line 64):
```python
if not hmac.compare_digest(token, api_key):
    raise HTTPException(status_code=401, detail="Invalid API key")
```

**MCP tools & resources** (`auth.py` line 56):
```python
if not hmac.compare_digest(api_key, expected):
    raise MCPAuthError("Invalid API key.")
```

Both layers use the same constant-time comparison. ✅

### MCP authentication — 100% coverage (8 tools + 4 resources) ✅

All 12 MCP entry points (8 tools + 4 resources) call `require_api_key()` as their first action. No authentication gaps remain. ✅

### Path confinement — dual-layer protection ✅

Filesystem path validation now uses:
1. `..` component rejection (raw + URL-encoded) — prevents traversal
2. Base-directory confinement via `os.path.commonpath()` — prevents absolute-path access

### Body size enforcement — unchanged from Iteration 2 ✅

Chunked encoding rejected with 413 before body read. 1 MiB default body limit. Both intact.

### Rate limiting — unchanged from Iteration 2 ✅

100 req/min default, health endpoint exempted, env-configurable. Intact.

### API key redaction — unchanged from Iteration 2 ✅

`_redact_sensitive_fields()` correctly redacts `api_key` in LLM provider responses. Intact.

---

## 04 · Remediation Recommendations

> **Must Fix**
> (none)

> **Should Fix**
> (none)

> **Consider**
> (none — all prior recommendations addressed)

---

## 05 · Security Coverage Summary

| Attack Surface | Status | Notes |
|---|---|---|
| Auth — REST API | ✅ Secured | `hmac.compare_digest()`, env-configured API key |
| Auth — MCP tools | ✅ Secured | All 8 tools call `require_api_key()` |
| Auth — MCP resources | ✅ Secured | All 4 resources now call `require_api_key()` |
| Timing-safe comparison | ✅ Consistent | Both REST and MCP use `hmac.compare_digest()` |
| Rate limiting | ✅ Enforced | 100 req/min, health exempt, configurable |
| Body size (chunked) | ✅ Blocked | 413 before body read |
| Body size (limit) | ✅ Enforced | 1 MiB default, env-configurable |
| API key leak prevention | ✅ Redacted | `_redact_sensitive_fields()` on LLM provider responses |
| Path traversal | ✅ Prevented | `..` rejection + base-directory confinement |
| Dependency audit | ✅ Clean | `pip-audit` finds no known vulnerabilities |
| Logging | ✅ Clean | No sensitive data logged; structured logging |

**Total Iteration 3 findings: 0** — All 3 Iteration 2 findings resolved. Full 608-test suite passes with zero regressions.

---

_Generated by Security Architect · 2026-07-26 · Validation Contract: 2026-07-25-contexter-phase3-python-layer · Iteration 3 (Final)_
