# Bug: Agent/Skill FTS Name Boost 1.5 vs Design's 2.0

**Severity:** LOW  
**Root Cause:** `agent_schema()` and `skill_schema()` set `name` field boost to `1.5` but the approved design preview specifies `name:2.0`.

## Requirements

### REQ-FIX-001: Fix agent name boost
In `fts/tantivy.rs` `agent_schema()`, change `name` field boost from `1.5` to `2.0`.

### REQ-FIX-002: Fix skill name boost
In `fts/tantivy.rs` `skill_schema()`, change `name` field boost from `1.5` to `2.0`.
