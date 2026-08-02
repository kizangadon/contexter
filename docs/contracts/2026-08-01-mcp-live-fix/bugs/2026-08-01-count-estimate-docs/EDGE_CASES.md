# EDGE CASES — Count Endpoints: Document Estimate Semantics

## EC-ED-001 — No false exactness claim
Wording MUST NOT claim unfiltered counts are exact after mutations — only fresh-store parity is exact. Use "estimate" language consistently.

## EC-ED-002 — Cross-reference precedent
The caveat applies to count_agents/count_skills too (accepted since PF-09) — the docs SHALL cover all three count endpoints, not just sessions.

## EC-ED-003 — Consistency with EC-CS-003
The documentation MUST not contradict the bug contract's documented accepted semantics (EC-CS-003) — same story, user-facing.

## EC-ED-004 — Filtered counts remain exact
Docs SHALL point to filtered counts (e.g., `count_sessions({"project": ...})`) and `list_*` tools as the exactness paths — but note list tools are bounded at 100 (PF-06), so exactness for large datasets is via filtered counts.

## EC-ED-005 — No code drift
If any validator later measures different numbers, docs remain directionally correct (order-of-magnitude behavior), not exact-forever claims.
