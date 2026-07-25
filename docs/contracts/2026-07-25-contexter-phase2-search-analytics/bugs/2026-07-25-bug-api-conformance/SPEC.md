# Bug: API/Design Deviations — Implementation Must Match Design Preview

**Severity:** MEDIUM  
**Root Cause:** Implementation deviated from the approved design preview in field names, trait methods, entity schemas, and cache policy. The design preview is immutable — implementation must conform.

## Requirements

### REQ-FIX-001: HybridSearchQuery field names
Rename fields to match the design preview:
- `text_query` → `query_text`
- `vector_query` → `query_vector`
- `limit` → `top_k`
- Add `text_weight` as a separate field (currently computed as `1.0 - vector_weight`)
- Remove `sort_field` and `agent_id` if they are not in the design preview
- Update all callers and tests

### REQ-FIX-002: FTS entity schemas
Implement entity-specific FTS schemas for session, agent, and skill (not just "memory" and "default"). Each schema should have appropriate fields and boosts per the design preview.

### REQ-FIX-003: Create memory cache policy
Change `create_memory` L1 cache policy from write-through to cache-invalidate to match the design preview.

### REQ-FIX-004: FTS field boosts
Adjust FTS memory schema boosts to match the design preview (content:1.0, tags:1.5 without the extra title:2.0 boost).
