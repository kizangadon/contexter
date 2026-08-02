# ACCEPTANCE — Bridge Double-Encode on Bytes Path

## AC-BD-001
GIVEN a store_memory call with content ≥102400 bytes
WHEN the memory is retrieved over live stdio
THEN the content is byte-identical to what was stored (no double-encoding corruption)

## AC-BD-002
GIVEN a search_memories query returning ≥102400-byte content
WHEN results are returned
THEN content is byte-identical

## AC-BD-003
GIVEN the fix
THEN full suite ≥647 passed / 1 known pre-existing; large-content round-trip test present
