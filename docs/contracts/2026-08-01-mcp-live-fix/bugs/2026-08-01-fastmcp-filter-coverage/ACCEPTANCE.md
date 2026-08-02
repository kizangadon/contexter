# ACCEPTANCE — FastMCP Filter Coverage: Complete Emitter/Prefix Coverage

## AC-FC-001 — All emitters covered
- **Given** the installed fastmcp 3.4.0 package
- **When** the filter's `_EMITTER_LOGGERS` and prefix set are checked against a live inventory of error-emitting logger sites
- **Then** every site (server tools, server resources, prompts, sampling) is covered by an emitter-logger AND a matching prefix

## AC-FC-002 — Schema-validation WARNING suppressed
- **Given** a tool call with invalid arguments (schema validation failure, FastMCP `Invalid arguments for tool` WARNING)
- **When** the server runs live
- **Then** stderr contains NO box characters, NO traceback, NO file:line reference, and the failure section is ≤400 bytes (comfortable margin below 512)

## AC-FC-003 — Prompt & sampling emitters covered (unit)
- **Given** a filter instance
- **When** records with the three prefixes are emitted from their TRUE originating loggers (`fastmcp.prompts.function_prompt`, `fastmcp.server.sampling.run`, `fastmcp.server.server`)
- **Then** all three are dropped; equivalent records from any logger in `_EMITTER_LOGGERS` are also dropped

## AC-FC-004 — No false suppression
- **Given** contexter's own logs (`contexter_server.core.bridge` `bridge_call_failed`, `handler_error`) and unrelated framework records
- **When** the filter processes them
- **Then** they pass through unchanged — bridge ERROR line still emitted, success-path stderr unchanged (INFO lifecycle only)

## AC-FC-005 — Drift test present and green
- **Given** the EFS test module
- **When** the drift/emitter-inventory test runs
- **Then** it enumerates the installed framework's emitter sites and passes; it would fail if a new uncovered emitter/prefix appeared

## AC-FC-006 — Drop-policy documented
- **Given** `fastmcp_logging.py`
- **When** its module docstring is read
- **Then** it documents the drop-policy and why dropping (not downgrading) is required, and a test asserts the policy holds

## AC-FC-007 — Suite green
- **Given** the full test suite
- **Then** `python -m pytest -q` shows 881 + new tests passed, 0 failures
