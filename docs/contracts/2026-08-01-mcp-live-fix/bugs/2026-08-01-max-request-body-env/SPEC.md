# SPEC — MAX_REQUEST_BODY Env Canonicalization (iter-1 finding SEC-F05)

## Context
Security validator found `MAX_REQUEST_BODY` env read lacks the canonical `CONTEXTER_` prefix
(literal REQ-EV-001 deviation; zero security impact).

## Requirements
- REQ-MRB-001: The env var read for max request body SHALL use the canonical `CONTEXTER_` prefix
  (e.g. `CONTEXTER_MAX_REQUEST_BODY`) — check where MAX_REQUEST_BODY is read (likely main.py /
  api deps) and canonicalize.
- REQ-MRB-002: Backward-compat: if any documented or tested env name exists, update tests/docs
  to the canonical name; the codebase SHALL contain zero non-canonical `MAX_REQUEST_BODY` reads.
- REQ-MRB-003: Behavior unchanged: default value and parsing identical; full suite passes.
