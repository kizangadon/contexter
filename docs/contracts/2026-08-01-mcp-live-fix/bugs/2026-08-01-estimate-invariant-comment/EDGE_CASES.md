# EDGE CASES — Estimate Fast Path: Document CF Invariant

## EC-EIC-001 — Keep comment honest
The comment MUST describe the actual invariant (sessions CF = session keys only; session_index CF = index entries), not a generic "estimate is approximate" note.

## EC-EIC-002 — Consistency across count endpoints
If count_agents/count_skills estimate paths lack the comment, add the same one — one convention for all three endpoints.

## EC-EIC-003 — No new dependencies
Comment only — no `cfg(test)` hooks, no new modules.
