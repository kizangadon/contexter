# EDGE_CASES — Camelization Live-Coverage Tests

| ID | Scenario | Expected |
|---|---|---|
| EC-CM-001 | Engine method returns unexpected field | Test fails with explicit field diff |
| EC-CM-002 | Engine method returns empty result | Parses; valid |
| EC-CM-003 | Engine method errors | Test surfaces structured error, not hang |
| EC-CM-004 | Method list grows (34+ new) | Harness enumerates dynamically or update documented |
