# EDGE CASES — count_memories Estimate Invariant Comment

## EC-IV-01 — Match sibling wording
Use the same terminology as the sibling `count_sessions`/`count_agents`/`count_skills` comments (fresh-CF language, `*_index` companion CF phrase) — do not invent a new formulation.

## EC-IV-02 — No adjacent region edits
Do not reformat or move the fast path code; the comment goes on the same lines where the sibling comments summarize the estimate behavior.

## EC-IV-03 — Comment accuracy for memories
`memories` entities do use a companion index CF — verify the sibling wording applies (it does: memory index). Do not claim false mechanisms (e.g., key count == table count) if outdated terminology differs.

## EC-IV-04 — Rust fmt / clippy unaffected
A comment cannot break `cargo fmt --check`; ensure no code whitespace changed either (comment-only diff).