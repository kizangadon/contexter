# ACCEPTANCE — count_memories Estimate Invariant Comment

## AC-IV-001 — Caveat present in count_memories fast path
- **Given** `contexter-core/src/storage/rocksdb.rs`
- **When** the `count_memories` estimate (`estimate_num_key`) fast path is read
- **Then** it carries an invariant caveat comment equivalent to the sibling functions: estimate valid on fresh CF / inflated after updates+deletes / valid ONLY because the CF holds exclusively entity keys with index entries in companion `*_index` CF

## AC-IV-002 — No behavior change
- **Given** the Rust suite
- **Then** `cd contexter-core && cargo test` passes 471+ tests / 0 failed — all count functions (including `count_memories` estimate and fallback) unchanged, test counts identical

## AC-IV-003 — Minimal diff
- **Given** `git diff`
- **Then** the change touches ONLY the comment region inside the `count_memories` estimate fast path block of `rocksdb.rs` — no other lines, no other files