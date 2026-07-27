# Acceptance Criteria

1. Search endpoint search results do not include embedding in `data` field
2. Session model constructs successfully with no agent_id
3. Test exists for embedding exclusion from model_dump
4. Test exists for naive datetime UTC coercion
5. Test exists for status "done" → "completed" normalization
6. All 610+ existing tests pass
