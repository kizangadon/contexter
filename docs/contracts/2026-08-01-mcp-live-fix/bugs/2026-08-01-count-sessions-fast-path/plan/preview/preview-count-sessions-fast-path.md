# Design Preview — Unfiltered `count_sessions` O(1) Fast Path

> Auto Bug Loop Iteration 3 · Bug contract: `2026-08-01-count-sessions-fast-path` · Finding: PF-10 (LOW)

## 1 · Architecture (as-is → to-be)

```mermaid
flowchart LR
    subgraph Python["Python (contexter-server)"]
        OV["AnalyticsService.get_overview<br/>(6 engine calls)"]
        BR["core/bridge.py<br/>count_sessions wrapper"]
    end

    subgraph Rust["Rust (contexter-core)"]
        ENG["Engine::count_sessions"]
        RB["RocksDbBackend::count_sessions"]
        CF["sessions CF<br/>(session keys only)"]
        IDX["session_index CF<br/>(index entries)"]
    end

    OV --> BR --> ENG --> RB
    RB -->|"unfiltered (no project)"| EST["estimate-num-keys<br/>O(1) CF property — NEW"]
    RB -->|"filtered (project)"| SCAN["index-prefix scan<br/>exact — UNCHANGED"]
    RB -.->|"fallback: property unavailable"| SCAN
    EST -.-> CF
    SCAN -.-> CF
    IDX -.-> SCAN
```

## 2 · Sequence (unfiltered count — new fast path)

```mermaid
sequenceDiagram
    participant S as get_overview
    participant B as Bridge
    participant E as Engine
    participant R as RocksDbBackend
    participant DB as RocksDB

    S->>B: count_sessions({})
    B->>E: count_sessions(filter=None)
    E->>R: count_sessions(None)
    R->>DB: get_property("rocksdb.estimate-num-keys", "sessions")
    DB-->>R: property value (O(1))
    R-->>E: count (u64)
    E-->>B: count
    B-->>S: count
```

## 3 · Behavior Contract

| Case | Filter | Path | Result |
|---|---|---|---|
| Unfiltered, property available | `{}` / `None` | estimate-num-keys O(1) | exact count on seeded stores (estimate semantics documented) |
| Unfiltered, property unavailable | `{}` / `None` | full scan fallback | exact count (unchanged behavior) |
| Filtered | `{"project": "X"}` | index-prefix scan | exact count (unchanged) |

## 4 · Verification Plan

1. Rust tests: unfiltered parity, empty → 0, filtered exactness, fallback (mirror `agent_skill_test.rs:148-262` pattern).
2. Rebuild wheel (`maturin build --release` + `pip3 install --user --break-system-packages --force-reinstall`).
3. Python tests: analytics overview counts, bridge live coverage against rebuilt wheel.
4. Full suite green (baseline 867 + new tests, 0 failures).
