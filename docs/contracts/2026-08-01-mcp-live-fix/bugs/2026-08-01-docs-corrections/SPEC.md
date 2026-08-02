# SPEC — Docs Corrections (iter-1 findings SEC-F03, DESIGN-OBS2, UT-4)

## Context
Three docs-only findings: (a) SEC-F03 — secret-bearing resource URIs + SSE gating undocumented
(doc gap); (b) DESIGN-OBS2 — design doc §7.4 claims camelCase cache telemetry but real Rust emits
snake_case (docs-only correction; implementation already reads real keys correctly); (c) UT-4 —
engine pre-lowercases memory content on write (Rust core REQ-S-003, out of scope) — SHALL be
documented so it is not flagged as an implementation gap.

## Requirements
- REQ-DOC-001: README/architecture SHALL document: resource URIs carrying `_api_key` and the
  SSE (server-sent events / auth) gating behavior.
- REQ-DOC-002: Design doc §7.4 telemetry table SHALL be corrected to snake_case keys as emitted
  by the real engine (with note that analytics layer maps to camelCase domain view).
- REQ-DOC-003: README SHALL document that engine stores memory content lowercased (REQ-S-003
  behavior) so 102400-B round-trip lowercasing is expected, not a bug.
