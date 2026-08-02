# ACCEPTANCE — FastMCP Framework Logging: Bounded Failure Stderr (End-to-End)

## AC-FL-001 — Engine failure stderr ≤512 chars, no traceback

- **Given** a live stdio MCP server against an engine that raises a mid-call error (e.g., invalid session id → ValueError)
- **When** a tool call fails (e.g., `get_session` with an invalid id)
- **Then** the TOTAL stderr emitted for that failure is ≤512 chars (bridge line + any framework output) and contains no raw traceback, no source frames, and no rich box-drawing characters (`╭`, `│`, `╰`)

## AC-FL-002 — All error classes covered

- **Given** the 9 error classes exercised in iter-2 (validation, auth, engine, not-found, storage, launch, etc.)
- **When** each is triggered through the live FastMCP path
- **Then** NO framework traceback box appears on stderr for any of them (0 boxes observed across the matrix)

## AC-FL-003 — Full diagnostics still in log file

- **Given** the same engine failure
- **When** the diagnostics log file is inspected (CONTEXTER_LOG_FILE / launch log)
- **Then** it still contains the full traceback for the failure (no loss of debuggability)

## AC-FL-004 — Client-visible frames unchanged

- **Given** auth failure and validation failure scenarios
- **When** the client observes responses
- **Then** `isError` structure and message text are byte-identical to pre-fix behavior (`'API key required...'`, `'Invalid API key.'`, structured not-found/validation messages) — no success smuggling

## AC-FL-005 — Success path & stdout purity unchanged

- **Given** healthy engine and successful calls
- **When** the server runs
- **Then** stderr at default level shows INFO lifecycle events only (no new noise), stdout stays pure JSON-RPC (5/5 frames parse), and the full suite is green (baseline 867 passed + new framework-EFS tests, 0 failures)

## AC-FL-006 — Launch failure still clean

- **Given** a corrupt engine directory
- **When** the server launcher runs
- **Then** rc=2, empty stdout, ONE clean stderr line with diagnostics path (unchanged), full traceback in launch log (unchanged)
