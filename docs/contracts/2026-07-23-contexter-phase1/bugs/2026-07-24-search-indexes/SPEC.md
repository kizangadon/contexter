# Bug 11: Search Performance — Add Secondary Indexes (Perf H2, M1, M4)

## Problem
Memory search does full CF scans with O(N) deserialization + keyword scoring. Pre-built indexes would make search O(log N).

## Fix Requirements
1. Add a secondary index CF `memory_index` in RocksDB
2. On memory create/update, write index entries: `session_id → [memory_id]`, `tags → [memory_id]`, `memory_type → [memory_id]`
3. search_memories uses indexes for filtered queries
4. For `count_*` methods, prefer `estimate-num-keys` RocksDB property over full scan
5. Pre-lowercase memory content on write for keyword search
