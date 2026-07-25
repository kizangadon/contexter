# Edge Cases — Bug-HNSW-Config

- EC-01: M=0 or M=1 — should be validated to minimum M=2
- EC-02: ef_construction < ef_search — valid but unusual; no validation needed
- EC-03: Very large ef_search (>1000) — accept but warn about performance
