# Design Preview — Count Endpoints: Document Estimate Semantics

> Auto Bug Loop Iteration 3 · Contract: `2026-08-01-count-estimate-docs` · Finding: PF-11 (LOW)

## 1 · Semantics to Document

```mermaid
flowchart LR
    subgraph Store["RocksDB store lifecycle"]
        SEED["fresh seed"] -->|"exact"| C1["count = actual"]
        UPD["updates (memtable history)"] -->|"inflates"| C2["count = 2× actual"]
        DEL["deletes"] -->|"inflates more"| C3["count = 3× actual"]
        FLUSH["flush()"] -->|"no correction"| C4["count still inflated"]
        COMP["compaction (eventual)"] -->|"corrects"| C5["count → actual"]
    end

    subgraph Doc["Documentation (README Design Decisions + arch spec §7.5)"]
        D1["estimate-num-keys semantics<br/>exact on fresh stores, inflates until compaction"]
        D2["measured example: 100 creates + 100 updates → 200 vs 100"]
        D3["flush() does NOT correct"]
        D4["exactness: filtered counts / list_* (bounded 100)"]
    end

    SEED -.-> D1
    UPD -.-> D1
    FLUSH -.-> D3
    C2 -.-> D2
```

## 2 · Acceptance Gates

| Check | Assertion |
|---|---|
| AC-ED-001 | README Design Decisions documents the caveat |
| AC-ED-002 | architecture spec §7.5 consistent caveat |
| AC-ED-003 | concrete measured numbers included |
| AC-ED-004 | docs-only change; suite green (881) |
