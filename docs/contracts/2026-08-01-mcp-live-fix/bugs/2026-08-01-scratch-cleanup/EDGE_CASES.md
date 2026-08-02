# EDGE_CASES — Scratch File Cleanup

| ID | Scenario | Expected |
|---|---|---|
| EC-SC-001 | Scratch file referenced by a test | Suite fails loudly → restore only the referenced one via Worker, not gitignore bypass |
| EC-SC-002 | Empty gitignored dirs | May remain; no files |
