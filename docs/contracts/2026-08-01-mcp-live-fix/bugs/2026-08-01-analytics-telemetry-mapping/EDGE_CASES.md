# EDGE_CASES — Analytics/Health Telemetry Mapping Repair

| ID | Scenario | Expected |
|---|---|---|
| EC-AN-001 | Empty engine analytics | Zero counts (valid), success |
| EC-AN-002 | Seeded engine analytics | Non-zero real counts |
| EC-AN-003 | Engine telemetry key absent | Explicit mapping/log, not silent zero |
| EC-AN-004 | Health with no engine ops yet | Graceful defaults |
| EC-AN-005 | Analytics without `_api_key` (key set) | Auth rejection preserved |
