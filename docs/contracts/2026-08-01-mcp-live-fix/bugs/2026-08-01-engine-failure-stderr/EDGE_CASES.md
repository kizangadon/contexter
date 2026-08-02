# EDGE_CASES — Engine-Failure Stderr Hygiene
- EC-EFS-001: multiple engine failures → each stderr line bounded independently.
- EC-EFS-002: launch failure path (exit 2) unchanged (already clean).
- EC-EFS-003: CLI (non-MCP) engine failure — stderr may show traceback (CLI is interactive; not MCP stdout purity).
- EC-EFS-004: DEBUG-level logging enabled → richer stderr allowed (opt-in).
