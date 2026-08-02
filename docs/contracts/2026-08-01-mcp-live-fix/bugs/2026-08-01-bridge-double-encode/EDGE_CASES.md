# EDGE_CASES — Bridge Double-Encode

| ID | Scenario | Expected |
|---|---|---|
| EC-BD-001 | Content exactly 102400 bytes | Bytes path; byte-identical |
| EC-BD-002 | Content 102399 bytes | String path unchanged |
| EC-BD-003 | Content 1MB | Bytes path; byte-identical; no perf cliff |
| EC-BD-004 | Content with multibyte UTF-8 | Byte-identical round-trip |
