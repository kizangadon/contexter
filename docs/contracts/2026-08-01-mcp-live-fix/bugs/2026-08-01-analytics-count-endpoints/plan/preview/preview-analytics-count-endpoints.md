# DESIGN PREVIEW — Analytics Count Endpoints
```mermaid
flowchart LR
    A[AnalyticsService.get_overview] -->|now| B[count_agents + count_skills]
    A -.->|before (scans)| X[list_agents 1M + list_skills 1M]
    B --> C[bridge.py count methods]
    C --> D[Rust engine count_agents/count_skills]
```
- Mirror existing count_sessions/count_memories in Rust + bridge.
- get_overview: replace scans with counts; keep _safe_get degradation.
