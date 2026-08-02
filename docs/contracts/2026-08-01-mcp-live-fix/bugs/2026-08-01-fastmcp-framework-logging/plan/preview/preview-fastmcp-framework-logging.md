# Design Preview — FastMCP Framework Logging: Bounded Failure Stderr (End-to-End)

> Auto Bug Loop Iteration 3 · Bug contract: `2026-08-01-fastmcp-framework-logging` · Finding: AC-EFS-001 (LOW)

## 1 · Problem Diagram (as-is)

```mermaid
flowchart LR
    subgraph Server["contexter-server (run_mcp.py)"]
        H["Handler raises<br/>HandlerError / MCPAuthError<br/>(ValueError subclasses)"]
        R["root stdlib logger<br/>INFO (only root configured)"]
    end

    subgraph Framework["FastMCP 3.4.0 (site-packages — read-only)"]
        G["generic except Exception<br/>server.py:1297"]
        L["logger.exception(...)<br/>exc_info=True"]
        F["fastmcp.* logger<br/>propagate=False, RichHandler→stderr"]
    end

    H --> G --> L --> F
    R -.->|"propagate=False — does NOT reach fastmcp"| F
    F -->|"2672-char rich traceback box"| STDERR["stderr<br/>(total 2897 bytes > 512) ❌"]
```

## 2 · Target (to-be)

```mermaid
flowchart LR
    subgraph Server2["contexter-server (run_mcp.py)"]
        H2["Handler raises<br/>HandlerError / MCPAuthError"]
        C["fastmcp logger configured<br/>(Option A: level/filter)<br/>OR errors subclass FastMCPError<br/>(Option B: exc_info=False)"]
        B["bridge bridge_call_failed<br/>224-char concise line ✅"]
        D["diagnostics log file<br/>full traceback (unchanged) ✅"]
    end

    H2 --> C -->|"no traceback box"| STDERR2["stderr<br/>(≤512 chars total) ✅"]
    H2 --> B --> D
    H2 -->|"structured isError frame<br/>(unchanged)"| CLIENT["client stdout"]
```

## 3 · Sequence (engine failure, Option B shown)

```mermaid
sequenceDiagram
    participant C as MCP Client
    participant FM as FastMCP
    participant H as Handler
    participant B as Bridge
    participant L as Logs

    C->>FM: tools/call get_session("not-a-uuid")
    FM->>H: call handler
    H->>B: get_session("not-a-uuid")
    B-->>H: raises ValueError (invalid session id)
    H-->>FM: raises HandlerError (FastMCPError subclass)
    FM->>FM: server.py:1284-1287 exc_info=False (no box)
    FM-->>C: isError=true structured frame
    Note over L: stderr: bridge 224-char line only<br/>log file: full traceback (3046 bytes)
```

## 4 · Acceptance Gates

| Check | Assertion |
|---|---|
| AC-FL-001 | engine failure total stderr ≤512 chars, no `╭` box |
| AC-FL-002 | 0 framework boxes across the 9 error classes |
| AC-FL-003 | diagnostics log still holds full traceback |
| AC-FL-004 | client-visible messages byte-identical |
| AC-FL-005 | success path + stdout purity + suite (867+) green |
| AC-FL-006 | launch failure rc=2, one clean line (unchanged) |
