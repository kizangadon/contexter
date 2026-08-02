# Design Preview — count_memories Estimate Invariant Comment

> Auto Bug Loop Iteration 6 · Contract: `2026-08-01-count-memories-invariant-comment` · Finding: Code Reviewer [LOW]

## 1 · Change Surface

```mermaid
flowchart LR
    A["rocksdb.rs count_memories<br/>estimate fast path (~L1029-1043)"] -->|"ADD comment"| B["invariant caveat:<br/>estimate valid only on fresh CF;<br/>inflated after updates/deletes;<br/>memories CF holds only entity keys<br/>(index entries in companion *_index CF)"]
    S["sibling count functions<br/>(sessions / agents / skills)"] -.->|"style reference,<br/>untouched"| B
```

## 2 · Acceptance Gates

| Check | Assertion |
|---|---|
| AC-IV-01 | caveat present in `count_memories` fast path, sibling-equivalent |
| AC-IV-02 | `cargo test` 471 / 0 failed, behavior unchanged |
| AC-IV-03 | diff confined to the comment block |