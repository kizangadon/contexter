# Edge Cases

- The AliasChoices change for session_id is backward-compatible — same inputs accepted as before
- Null agent_id test verifies both "missing" and "explicit None" paths
- role=None test verifies Optional field behavior (None ≠ default)
