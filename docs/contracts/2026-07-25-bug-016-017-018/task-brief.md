# Task Brief: Security Hardening — BUG-016, BUG-017, BUG-018

## Context

Three low-severity security hardening fixes in the Contexter Python server (`contexter-server/`). All changes are small, independent, and well-defined.

---

## BUG-016a: Chunked Encoding Bypass

**File:** `contexter-server/src/contexter_server/main.py`

**Problem:** The body size limiter middleware (`_add_body_size_limit_middleware`) only checks `Content-Length` header. A `Transfer-Encoding: chunked` request bypasses the limit entirely.

**Fix:** Add rejection of `Transfer-Encoding: chunked` (return 413 with `{"detail": "Transfer-Encoding chunked not supported"}`).

Location: In `_add_body_size_limit_middleware`, before the Content-Length check, check `request.headers.get("Transfer-Encoding", "")` — if it contains `"chunked"`, return 413 immediately.

## BUG-016b: Default MAX_REQUEST_BODY Too High

**File:** `contexter-server/src/contexter_server/main.py`

**Problem:** Default is `50 * 1024 * 1024` (50 MiB). Too permissive.

**Fix:** Change default to `1 * 1024 * 1024` (1 MB).

Location: In `_add_body_size_limit_middleware`, change `str(50 * 1024 * 1024)` to `str(1 * 1024 * 1024)`.

Also update the docstring that says `"52 428 800 (50 MiB)"` → `"1 048 576 (1 MiB)"`.

## BUG-017: Timing-Safe API Key Comparison

**File:** `contexter-server/src/contexter_server/api/deps.py`

**Problem:** Line 63 uses `token != api_key` which is a string comparison vulnerable to timing attacks.

**Fix:** Replace `token != api_key` with `hmac.compare_digest(token, api_key)` and add `import hmac` at the top of the file.

## BUG-018: File Diff Path Validation TODO

**File:** `contexter-server/src/contexter_server/api/files.py`

**Problem:** The `file_diff` endpoint has `base` and `compare` query params that are passed through without path validation.

**Fix:** Add `# TODO: validate base/compare with validate_safe_path()` as a comment before the stub return statement in the `file_diff` function.

---

## Acceptance Criteria

- `hmac.compare_digest` is used in `deps.py` (not `!=`)
- Chunked encoding is rejected with 413 in `main.py`
- Default `MAX_REQUEST_BODY` is 1 MB (not 50 MB)
- TODO comment added in `files.py` for diff path validation
- All 543+ tests pass
- At least 1 new test for the body size changes (chunked encoding rejection)

## Skills to Load

`handoff`, `clean-code`, `git-workflow-and-versioning`, `verification-before-completion`, `incremental-implementation`, `tdd`, `domain-driven-design`, `python-pro`, `secure-code-guardian`, `security-and-hardening`, `test-driven-development`, `python-testing-patterns`

## Implementation Order (TDD)

1. Write test(s) first:
   - Add test for chunked encoding rejection to `test_security.py` (in `TestBodySizeLimit`)
   - Verify the existing body size test still passes with the new default (1 MB)
2. Implement fixes in `main.py`, `deps.py`, `files.py`
3. Run tests, verify all pass
4. Return Handoff Report
