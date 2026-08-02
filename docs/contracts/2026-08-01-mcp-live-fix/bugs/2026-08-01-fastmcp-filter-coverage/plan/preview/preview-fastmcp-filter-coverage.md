# Design Preview — FastMCP Filter Coverage: Complete Emitter/Prefix Coverage

> Auto Bug Loop Iteration 3 · Contract: `2026-08-01-fastmcp-filter-coverage` · Findings: Code F-1/F-6, Security F-IT3-01, UT MEDIUM

## 1 · Emitter Inventory (fastmcp 3.4.0 — installed framework)

```mermaid
flowchart LR
    subgraph EM["Error-emitting logger sites (fastmcp 3.4.0)"]
        S["fastmcp.server.server<br/>'Error calling tool …' :1297"]
        R["fastmcp.server.server<br/>'Error reading resource …'"]
        W["fastmcp.server.server<br/>'Invalid arguments for tool …' :1290 WARNING"]
        P["fastmcp.prompts.function_prompt<br/>'Error rendering prompt …' :370"]
        M["fastmcp.server.sampling.run<br/>'Error calling sampling tool …'"]
    end

    subgraph F["_SuppressFrameworkTracebackBox filter (fastmcp_logging.py)"]
        L["_EMITTER_LOGGERS<br/>= all 5 emitter loggers (complete)"]
        PX["prefixes<br/>= all 5 messages (explicit, no collisions)"]
    end

    S --> F
    R --> F
    W --> F
    P --> F
    M --> F
    F -->|"drop (all levels, exc_info or not)"| OK["stderr: 0 boxes<br/>validation ≤400B"]
```

**Gap today:** `P` and `M` uncovered; `W` prefix unmatched → validation-class stderr 486B (95% budget), 567B width-dependent.

## 2 · Drift Test (pins coverage)

```mermaid
sequenceDiagram
    participant T as Drift test
    participant PKG as installed fastmcp package
    participant F as Filter config

    T->>PKG: enumerate error-emitting logger sites + message prefixes
    T->>F: read _EMITTER_LOGGERS + prefixes
    alt every site covered
        F-->>T: PASS
    else new/uncovered site found
        T-->>T: FAIL (fail loudly, no silent box leak)
    end
```

## 3 · Acceptance Gates

| Check | Assertion |
|---|---|
| AC-FC-001 | all 5 emitters + prefixes covered (inventory-verified) |
| AC-FC-002 | validation failure stderr ≤400B, 0 box/traceback/file:line |
| AC-FC-003 | prompt/sampling/validation records dropped from true origin loggers |
| AC-FC-004 | contexter logs unaffected (bridge line still emitted) |
| AC-FC-005 | drift test green; fails on uncovered emitter |
| AC-FC-006 | drop-policy documented + asserted |
| AC-FC-007 | suite green (881 + new) |
