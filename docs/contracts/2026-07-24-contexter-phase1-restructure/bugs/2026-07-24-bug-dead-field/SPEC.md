# Bug 12: Dead Field — MemorySearchQuery.project

## REQ-DED-001: Remove or mark unused MemorySearchQuery.project
The `MemorySearchQuery.project` field (models/memory.rs:85) is never used in `search_memories` or `resolve_memory_ids_via_index`. The associated comment says "`project` filter skipped — Memory does not carry a project field."

**Fix**: Add `#[serde(skip)]` to the field and a `#[allow(dead_code)]` attribute explaining it's reserved for Phase 2 when Memory resolves project via Session join. This way the field is still accepted (and silently ignored) in search query JSON, but the compiler warning is suppressed.
