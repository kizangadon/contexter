# SPEC — FastMCP Filter Coverage: Complete Emitter/Prefix Coverage

> Parent: `2026-08-01-mcp-live-fix` · Auto Bug Loop Iteration 3
> Findings: Code Reviewer F-1 (LOW), Security F-IT3-01 (LOW), User-Testing MEDIUM (schema-validation WARNING), Code Reviewer F-6 (INFO — drop-policy pin)

## Problem

The `_SuppressFrameworkTracebackBox` filter in `contexter-server/src/contexter_server/fastmcp_logging.py` covers `("fastmcp", "fastmcp.server", "fastmcp.server.server")` and the prefixes `Error calling tool …`, `Error reading resource …`, `Error rendering prompt …`. Coverage is INCOMPLETE vs its documented scope:

1. `fastmcp/prompts/function_prompt.py:370` emits `"Error rendering prompt …"` on logger `fastmcp.prompts.function_prompt` — NOT in `_EMITTER_LOGGERS` (Python applies only originating-logger filters → the box would leak if prompts were registered). **Latent today** (contexter registers zero prompts) — but REQ-FL-001 regression risk.
2. `fastmcp/server/sampling/run.py` emits `"Error calling sampling tool …"` — logger not covered AND the prefix `"Error calling tool "` does not match (the word "sampling" breaks the prefix match).
3. `fastmcp/server/server.py:1290` schema-validation WARNING `"Invalid arguments for tool …"` (with `exc_info`) is NOT suppressed → validation-class failure stderr measured **486B (95% of the 512B budget)**, width-dependent (567B inclusive of padded startup marker), containing a file:line reference. Letter passes; margin fragile.
4. The drop-policy (records dropped at every level, including `e.log_level` paths) is not explicitly documented/pinned.

## Requirements

### REQ-FC-001 — Complete emitter coverage
The filter SHALL cover EVERY fastmcp 3.4.0 logger that emits one of the framework error messages listed above (incl. `fastmcp.prompts.function_prompt` and `fastmcp.server.sampling.run`), based on an actual inventory of the installed framework (all emitter sites).

### REQ-FC-002 — Complete prefix coverage
The filter SHALL match ALL framework error/warning messages that carry traceback or file:line content, including `Error calling sampling tool …` and the schema-validation WARNING `Invalid arguments for tool …` (server.py:1290). Matching SHALL be explicit per-prefix (no accidental substring collisions, no false suppression of contexter's own logs).

### REQ-FC-003 — Validation-class margin
Validation-class failure stderr SHALL have a comfortable margin below 512 bytes — target ≤400 bytes measured live, including the width-dependent padded-startup-marker scenario measured by User-Testing (567B inclusive). 0 box chars, 0 file:line references in stderr.

### REQ-FC-004 — Drift test (emitter inventory)
A regression test SHALL enumerate the installed fastmcp package's error-emitting logger sites (at least: the three server emitters, the prompt emitter, the sampling emitter) and FAIL if a new emitter site or message prefix appears that is not covered by the filter — pinning both `_EMITTER_LOGGERS` and the prefix set.

### REQ-FC-005 — Drop-policy documented and pinned
The filter module docstring SHALL document the drop-policy (records dropped at all levels for the listed messages; downgrade proven insufficient for long payloads — 583B wrap measured) and the drift test SHALL assert the policy holds (e.g., a WARNING-level record with `exc_info` for the covered messages is dropped, while unrelated records pass).

## Non-Goals

- No change to contexter's own bridge/handler logging (still emits `bridge_call_failed` / `handler_error`).
- No client-visible frame changes.
- No framework source edits.

## Artifacts

- Contract dir: `docs/contracts/2026-08-01-mcp-live-fix/bugs/2026-08-01-fastmcp-filter-coverage/`
- References: `plan/review/review-mcp-live-fix-scrutiny-code-review-iter-3.md` (F-1, F-6), `...-scrutiny-security-review-iter-3.md` (F-IT3-01), `...-user-testing-review-iter-3.md` (MEDIUM finding)
