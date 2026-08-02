# ACCEPTANCE — Bridge Log Hygiene

## AC-BH-001
GIVEN a store_memory call with a 10KB content string
WHEN the bridge logs args_summary
THEN the logged content prefix is ≤ the documented cap (64 chars) and full content never appears in any log line

## AC-BH-002
GIVEN the summary function
THEN unit tests assert the cap for content, query, and any other content-bearing args

## AC-BH-003
GIVEN the fix
THEN full suite ≥647 passed / 1 known pre-existing
