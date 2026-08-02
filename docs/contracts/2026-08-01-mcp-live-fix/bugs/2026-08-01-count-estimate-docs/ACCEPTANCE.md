# ACCEPTANCE — Count Endpoints: Document Estimate Semantics

## AC-ED-001 — README caveat present
- **Given** the README Design Decisions section
- **When** it is read
- **Then** it documents that unfiltered count endpoints use `estimate-num-keys`: exact on fresh stores, inflated after updates/deletes until compaction, `flush()` does not correct, exactness via filtered counts / list tools

## AC-ED-002 — Architecture spec caveat present
- **Given** `docs/design/specs/2026-07-23-contexter-system-architecture.md` (count-endpoints section)
- **When** it is read
- **Then** it carries the same caveat, consistent with the README

## AC-ED-003 — Concrete measured numbers
- **Given** the documentation
- **When** the inflation magnitude is described
- **Then** it includes at least one concrete measured example (e.g., 100 creates + 100 updates → count 200 vs 100 actual)

## AC-ED-004 — Docs-only, suite green
- **Given** the full test suite
- **Then** `python -m pytest -q` shows 881 passed / 0 failures and no implementation or test file changed (only README.md and the architecture spec)
