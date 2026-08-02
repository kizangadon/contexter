# Design Preview — Estimate Fast Path: Document CF Invariant

> Auto Bug Loop Iteration 3 · Contract: `2026-08-01-estimate-invariant-comment` · Finding: Code F-5 (INFO)

## 1 · Invariant to Document

```mermaid
flowchart LR
    CF["sessions CF<br/>(session keys ONLY)"] -->|"estimate-num-keys valid"| EST["O(1) count"]
    IDX["session_index CF<br/>(index entries)"] -->|"filtered scans only"| SCAN["exact count"]
    note["COMMENT: estimate valid ONLY because CF holds exclusively keys.<br/>If invariant breaks, unfiltered counts must not use the estimate."]
```

## 2 · Acceptance Gates

| Check | Assertion |
|---|---|
| AC-EIC-001 | invariant comment at estimate paths (sessions + agents/skills if missing) |
| AC-EIC-002 | zero logic changes; suites green |
