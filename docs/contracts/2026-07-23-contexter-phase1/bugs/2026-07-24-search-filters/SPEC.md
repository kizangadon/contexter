# Bug 15: SPEC REQ-E-005 Search Filters

## Problem
Engine's `search_memories()` bypasses RocksDbBackend's `search_memories()` (which uses the new memory_index CF) and does its own chunked scan. It only implements keyword scoring and agent_id filtering, but not memory_type, tags, or session_id filters that the `MemorySearchQuery` struct defines.

## Fix Requirements
1. Make Engine's `search_memories()` delegate to `RocksDbBackend::search_memories()` or implement the missing filters
2. Ensure memory_type, tags, and session_id filters from MemorySearchQuery work end-to-end
3. Integration tests should verify each filter works
