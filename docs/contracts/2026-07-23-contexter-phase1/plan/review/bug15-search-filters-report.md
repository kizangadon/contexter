# Bug 15: Search Filters — Fix Report

**Date:** 2026-07-24
**Contract:** `docs/contracts/2026-07-23-contexter-phase1/bugs/2026-07-24-search-filters/`
**SPEC:** `SPEC.md` (REQ-E-005)

---

## Problem

Engine's `search_memories()` was performing its own chunked scan of the
`CF_MEMORY_ITEMS` column family, duplicating logic that already existed in
`RocksDbBackend::search_memories()`. The Engine implementation only supported
`keywords` and `agent_id` filters, silently ignoring `memory_type`, `tags`, and
`session_id` fields from `MemorySearchQuery`.

The existing test `test_memory_create_and_search` happened to pass because the
ignored tag filter caused *all* memories to be returned — the assertion
`.len() == 1` was coincidentally correct.

---

## Fix Applied

**Approach A** — Delegate from Engine to RocksDbBackend.

### Change: `src/engine/mod.rs`

**Before:** `Engine::search_memories()` did its own chunked scan of
`CF_MEMORY_ITEMS` with hardcoded keyword scoring and `agent_id` filtering.
It held the `SharedBackend` read lock briefly per chunk but never consulted
the `memory_index` CF.

**After:** `Engine::search_memories()` delegates directly to
`self.storage.read().unwrap().search_memories(query)`, which calls through to
`RocksDbBackend::search_memories()`. The backend implementation:

1. Uses `resolve_memory_ids_via_index()` to intersect secondary index lookups
   for `session_id`, `memory_type`, and `tags` filters from the `memory_index` CF
2. Falls back to full scan for keyword-only queries
3. Applies keyword relevance scoring (multi-keyword, with exact/starts/contains tiers)
4. Applies `agent_id` filter post-scan
5. Sorts by relevance (desc) then `updated_at` (desc)
6. Applies offset/limit

### Changes to imports

Removed unused `CF_MEMORY_ITEMS` and `KEY_PREFIX_MEMORY` imports.

### New tests added in `src/engine/mod.rs`

| Test | What it verifies |
|------|-----------------|
| `test_search_by_memory_type` | Creates Fact + Preference memories, searches by Fact (1 result) and Episode (0 results) |
| `test_search_by_session_id` | Creates memories in two sessions, searches each (1 result each) |
| `test_search_by_tags` | Creates memories with overlapping tags, searches by unique tag (1 result), shared tag (2 results), non-matching tag (0 results) |
| `test_search_combined_filters` | Tests `session_id + type`, `session_id + tag`, `session_id + type + tag`, and `session + non-matching-tag` combinations |

---

## Verification

### All 181 unit tests pass
```
test result: ok. 181 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### All 13 integration tests pass
```
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Clippy passes with `-D warnings`
```
Finished dev profile [unoptimized + debuginfo] in 1.66s
```

### All search-specific tests (9) pass
- `engine::tests::test_memory_create_and_search` — keyword + tag (existing)
- `engine::tests::test_search_by_memory_type` — **new**
- `engine::tests::test_search_by_session_id` — **new**
- `engine::tests::test_search_by_tags` — **new**
- `engine::tests::test_search_combined_filters` — **new**
- `storage::rocksdb_backend::tests::test_memory_search_keyword` — existing backend test
- `storage::rocksdb_backend::tests::test_memory_search_filters` — existing backend test

---

## Design Rationale

The `memory_index` column family already existed with secondary index entries
for `session_id`, `memory_type`, and `tags`. The Engine was duplicating the
search logic without using these indexes. Delegating to the backend:

1. **Removes code duplication** — the search logic lives in one place
2. **Enables indexed lookups** — `memory_type`, `tags`, and `session_id`
   filters now use the `memory_index` CF via prefix scans
3. **Preserves correctness** — keyword scoring and `agent_id` filtering still
   work, now with the full filter set

The trade-off is that the `SharedBackend` read lock is now held for the
duration of the backend call, rather than released between chunks. This is
the same contract used by `count_memories` and other bypass-policy methods.
For indexed queries the scan is fast; for full keyword scans the existing
backend implementation uses a single RocksDB iterator (no chunking), which is
the same approach used by `list_sessions`, `list_agents`, etc.
