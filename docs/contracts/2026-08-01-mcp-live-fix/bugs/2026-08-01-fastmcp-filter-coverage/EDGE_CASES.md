# EDGE CASES — FastMCP Filter Coverage: Complete Emitter/Prefix Coverage

## EC-FC-001 — Origin-logger filters only
Python logging applies filters on the ORIGINATING logger only (not parents) — every emitter logger MUST be listed explicitly in `_EMITTER_LOGGERS` (incl. `fastmcp.prompts.function_prompt`, `fastmcp.server.sampling.run`).

## EC-FC-002 — Prefix collision safety
`"Error calling tool "` must not be matched via a loose substring in a way that suppresses unrelated messages, nor miss `"Error calling sampling tool "`. Use explicit per-prefix matching (e.g., `startswith` with the exact prefix list including the sampling variant).

## EC-FC-003 — WARNING-level schema-validation record
The `Invalid arguments for tool …` record is a WARNING with `exc_info` from `server.py:1290` — the filter must drop it at WARNING level too (drop-policy applies at all levels).

## EC-FC-004 — FastMCP version drift
If a future fastmcp adds an emitter (e.g., new feature), the drift test MUST fail loudly rather than silently leak boxes. Keep the drift test enumerating installed package sources, not hard-coded strings only.

## EC-FC-005 — Downgrade insufficiency
Do NOT regress to level-downgrade semantics: measured 583B wrap for long payloads on `resource_read` — the drop-policy is the contract (REQ-FC-005).

## EC-FC-006 — Filter idempotence & survival
The filter remains idempotent per logger and survives fastmcp `configure_logging` (removes handlers only) — drift tests must not break these properties.

## EC-FC-007 — No content leak via new prefixes
Suppressing the schema-validation WARNING must not cause argument content to appear elsewhere on stderr — mirror the 10KB-content no-leak assertion (AC-BH-001 pattern).
