# ACCEPTANCE — Unfiltered `count_sessions` O(1) Fast Path

## AC-CS-001 — Unfiltered count returns exact count (parity)

- **Given** a fresh engine store with N sessions seeded (and M memories/agents/skills interleaved)
- **When** `count_sessions({})` (no filter) is called
- **Then** it returns N, matching the count obtained by listing sessions — exact parity with the filtered/scan semantics

## AC-CS-002 — Empty store returns 0

- **Given** a fresh empty store
- **When** `count_sessions({})` is called
- **Then** it returns 0

## AC-CS-003 — Filtered count unchanged

- **Given** sessions spanning multiple projects
- **When** `count_sessions({"project": "X"})` is called
- **Then** it returns exactly the sessions belonging to project X (index-prefix scan semantics preserved; no regression)

## AC-CS-004 — Latency flat across store growth

- **Given** a store with 2,000+ sessions
- **When** unfiltered `count_sessions({})` is measured against the empty-store baseline
- **Then** latency does not scale with store size (no per-row serde deserialization; consistent with the flat measurements of count_agents/count_skills) — e.g., sub-millisecond at 2,000 sessions, versus the previous 2.5 ms

## AC-CS-005 — get_overview correct end-to-end

- **Given** the MCP analytics overview resource / CLI status path
- **When** `get_overview` runs against a store with known session count
- **Then** the reported session count matches the engine counters, and the full test suite is green (baseline 867 passed + new tests, 0 failures)

## AC-CS-006 — Fallback preserved

- **Given** the `estimate-num-keys` CF property is unavailable (fallback condition)
- **Then** the exact full-scan path returns the correct count (same fallback semantics as count_agents/count_skills)
