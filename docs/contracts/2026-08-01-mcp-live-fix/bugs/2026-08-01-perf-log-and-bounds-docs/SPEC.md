# SPEC — Perf Log Level & Bounds Documentation (iter-1 findings PERF-PF05..PF08)

## Context
Performance validator re-stated 4 informational items: (a) per-call INFO logging (PF-05),
(b) list tools bounded at 100 with no pagination (PF-06), (c) store_memory 2 sequential calls
deliberate (PF-07), (d) export 10k/entity bounded + LRU-cached (PF-08). Items (b)-(d) are
deliberate, documented design decisions — SHALL be explicitly documented as accepted decisions in
README/architecture. Item (a) — per-call INFO logging — SHALL be addressed so validators stop
flagging it.

## Requirements
- REQ-PLB-001: Per-call INFO logging in hot paths (bridge/handlers per-call logs) SHALL be
  reclassified: keep INFO for lifecycle/errors; move per-call request logging to DEBUG (or keep
  INFO ONLY where observability contract REQ-HO-002 explicitly requires INFO). Verify REQ-HO-002
  text; do not weaken required observability.
- REQ-PLB-002: README + architecture doc SHALL contain explicit "Accepted performance
  decisions" section: 100-item list cap (no pagination), sequential store_memory calls,
  10k/entity export cap + LRU cache — each with rationale.
- REQ-PLB-003: Tests still pass; grep shows per-call INFO logs moved to DEBUG where not
  contract-required.
