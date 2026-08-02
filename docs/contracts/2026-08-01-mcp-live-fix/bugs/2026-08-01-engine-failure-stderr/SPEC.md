# SPEC — Engine-Failure Stderr Hygiene (iter-1 finding UT-2, INFO)

## Context
User-Testing flagged: stderr receives rich tracebacks for engine-level failures at
bridge.py:181 `logger.exception` — bounded (≤64-char args, max stderr line 337 chars, no content
leak) and stdout unaffected, but contradicts the strictest reading of the launch/error contract.
Resolve by making the diagnostics destination explicit and consistent with the launch-failure
design (launch log file).

## Requirements
- REQ-EFS-001: Engine-failure diagnostics during MCP runtime SHALL log the full exception to a
  bounded diagnostics channel (the launch log file or structlog-explicit error line) rather than
  an unbounded `logger.exception` traceback to stderr — while KEEPING a single concise
  structured stderr line (kind + bounded context) so operator visibility is preserved.
- REQ-EFS-002: stderr output for engine failures SHALL remain bounded (< 512 chars per failure).
- REQ-EFS-003: stdout SHALL remain pure (no change).
- REQ-EFS-004: Tests: engine-failure scenario asserts bounded stderr (no full traceback), full
  diagnostics available via log file, existing 794-suite behavior unchanged.
