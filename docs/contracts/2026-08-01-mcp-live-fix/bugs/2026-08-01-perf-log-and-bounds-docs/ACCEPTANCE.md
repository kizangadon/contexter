# ACCEPTANCE — Perf Log & Bounds Docs
- AC-PLB-001: GIVEN grep of per-call request logs, THEN they use DEBUG level unless REQ-HO-002 requires INFO.
- AC-PLB-002: GIVEN README/architecture, THEN an "Accepted performance decisions" section exists covering list cap 100, sequential store_memory, export 10k + LRU.
- AC-PLB-003: WHEN full suite runs, THEN 0 failures.
