# SPEC — Handler ID Bounding (iter-1 findings)

## Context
Iteration-1 validators found: (a) `not_found_error(id)` echoes unbounded caller-controlled ids in
non-UUID-validated handlers — empirically 1MB id → 1,000,020-char error message (violates
REQ-IV-005/EC-IV-009); (b) handler log bindings carry raw unbounded ids — 1MB request inflates
log lines (violates REQ-HO-002/B9 bounds). `_bounded(id)` helper exists but is not applied in
these paths.

## Requirements
- REQ-HIB-001: `not_found_error(id)` SHALL bound echoed ids via the existing `_bounded()` helper
  in ALL handlers (agent_id, session_id, skill_id paths at handlers.py L168/254/317/442/470/497).
- REQ-HIB-002: All handler log bindings carrying request ids SHALL pass ids through `_bounded()`
  (L146/241/275/304/332/429/489 area) so a 1MB id produces ≤64-char log content.
- REQ-HIB-003: Error message text SHALL remain byte-identical for ids ≤64 chars (no behavior
  change for legitimate inputs).
- REQ-HIB-004: Tests SHALL prove: 1MB id → error message ≤ 256 chars total; log payload for 1MB
  id ≤ 64 chars for the id field; valid 36-char UUID ids unaffected.
