# Acceptance Criteria

1. session_id accepts both "sessionId" and "session_id" from JSON input
2. embedding is excluded from model_dump() and model_dump_json() output
3. embedding is still accepted as input (deserialization unchanged)
4. Naive datetime "2024-01-01T00:00:00" is coerced to "2024-01-01T00:00:00Z" (UTC)
5. Session status "done" is normalized to "completed"
6. All 610+ existing tests pass
