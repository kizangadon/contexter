# EDGE_CASES — Env Var Canonicalization

| ID | Scenario | Expected |
|---|---|---|
| EC-EV-001 | Both typo and canonical set | Canonical wins (explicit precedence) |
| EC-EV-002 | Only typo set | Canonical default applies; deprecation log if alias retained |
| EC-EV-003 | Neither set | Default pool size |
| EC-EV-004 | Canonical set to 0/invalid | Clamp to default; no crash |
