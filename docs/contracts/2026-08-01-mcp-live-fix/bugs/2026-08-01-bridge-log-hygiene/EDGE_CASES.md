# EDGE_CASES — Bridge Log Hygiene

| ID | Scenario | Expected |
|---|---|---|
| EC-BH-001 | Content exactly at cap | Logged fully (≤cap) |
| EC-BH-002 | Content beyond cap | Truncated to cap |
| EC-BH-003 | Empty content | Summary shows placeholder, no leak |
| EC-BH-004 | Non-ASCII/multibyte content | Cap by chars, no encoding error |
