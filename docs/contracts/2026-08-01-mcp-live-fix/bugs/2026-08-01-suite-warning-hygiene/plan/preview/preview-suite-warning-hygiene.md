# Design Preview — Suite Warning Hygiene

> Auto Bug Loop Iteration 3 · Contract: `2026-08-01-suite-warning-hygiene` · Finding: SPEC INFO (starlette PendingDeprecationWarning)

## 1 · Decision Path

```mermaid
flowchart TD
    WARN["1 warning: starlette PendingDeprecationWarning<br/>(python-multipart)"] --> ID["identify exact source"]
    ID --> UP["dependency bump removes it?"]
    UP -->|"yes"| FIX["upgrade/pin — cleanest, no filter debt"]
    UP -->|"no"| FILT["scoped filterwarnings entry<br/>module + class + message-scoped<br/>with justification comment"]
    FILT -->|"forbidden"| BLANK["blanket -W ignore ❌"]
```

## 2 · Acceptance Gates

| Check | Assertion |
|---|---|
| AC-SW-001 | suite: 0 warnings, 881+ passed |
| AC-SW-002 | filter (if used) scoped + justified |
| AC-SW-003 | other warnings still surface |
| AC-SW-004 | suite green, test content unchanged |
