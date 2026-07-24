# Bug: Entity field discrepancies from SPEC

## Problem
Two entity structs don't match the SPEC/design-preview field definitions:

1. `models/session.rs`: Missing `efficiency_score: Option<f64>` field
2. `models/audit.rs`: Uses `changes` (not `summary`), `timestamp` (not `created_at`), and missing `metadata: HashMap<String, String>`

## Requirements
- REQ-001: Add `efficiency_score: Option<f64>` to Session struct
- REQ-002: Rename `AuditEntry.changes` → `AuditEntry.summary` (String type, same semantics)
- REQ-003: Rename `AuditEntry.timestamp` → `AuditEntry.created_at` (DateTime<Utc> type)
- REQ-004: Add `metadata: HashMap<String, String>` to AuditEntry struct
- REQ-005: Update all references to the old field names throughout the codebase
- REQ-006: `cargo build && cargo test` must pass after changes
