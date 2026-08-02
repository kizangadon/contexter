# SPEC — Launcher Exception Type Pin (iter-1 findings UT-AC-TH-001, SPEC-REQ-TH)

## Context
`tests/mcp/test_mcp_launcher_wiring.py:218` uses bare `pytest.raises(Exception)`. User-Testing
verified the engine raises a STABLE `RuntimeError` on corrupt data dir, so the precise type is
catchable. SPEC validator notes the contract (REQ-TH-001/003, AC-TH-001) has no carve-out for
broad matching.

## Requirements
- REQ-LET-001: Pin `pytest.raises(Exception)` at test_mcp_launcher_wiring.py:218 to the precise
  exception type the engine reliably raises (RuntimeError), keeping the test's documented intent.
- REQ-LET-002: Verify empirically the pinned type matches the real engine behavior (corrupt dir
  scenario); document why in a comment.
- REQ-LET-003: Full suite passes; no other broad `pytest.raises(Exception)` remains repo-wide.
