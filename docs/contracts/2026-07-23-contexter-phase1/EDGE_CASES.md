# Edge Cases — Contexter Phase 1: Rust Core Foundation

---

## Feature Overview

Phase 1 delivers the Rust core engine with RocksDB storage, hot cache, PyO3 bridge, and CLI. The main flows are: engine initialization, entity CRUD (session, memory, agent, skill), cache operations, key-value store, maintenance operations (checkpoint, storage_size), and CLI diagnostics. Edge cases cover storage initialization failures, data corruption, concurrency, boundary values, and cross-tier interaction.

---

## Edge Case Categories

### 1. Storage Initialization Edge Cases

| ID | Scenario | Trigger | Expected Behavior | Priority |
|---|---|---|---|---|
| E-001 | Data directory doesn't exist | Engine init with nonexistent path | RocksDB creates the directory automatically (`create_if_missing=true`) | High |
| E-002 | Data directory exists but is empty | Fresh install, first run | Engine opens successfully, creates column families, all counts are zero | High |
| E-003 | Data directory contains a file named same as expected RocksDB dir | Path collision | RocksDB returns IOError → Engine returns error with descriptive message | Medium |
| E-004 | Data directory is a file, not a directory | `~/.contexter` is a regular file | RocksDB open fails → Engine returns error | Medium |
| E-005 | Data directory is on a read-only filesystem | Permissions issue | RocksDB open fails → Engine returns "path not writable" error | High |
| E-006 | Data directory has insufficient disk space (< 10MB) | Disk full or quota | RocksDB may open but writes will fail with IOError. Engine should catch and return resource-exhausted error | Medium |
| E-007 | Data directory path contains unicode characters | Non-ASCII path | RocksDB handles UTF-8 paths on Linux. Engine passes through correctly | Low |
| E-008 | Data directory path is relative vs absolute | `./contexter_data` | Engine should canonicalize to absolute path | Low |
| E-009 | config.yaml missing or unreadable | First run, no config file | Engine uses defaults (path: ~/.contexter/, cache: 10K entries/type) | High |
| E-010 | MANIFEST file corrupted on open | Crash during previous write | RocksDB detects corruption → Engine returns corruption error with message suggesting recovery from WAL | High |
| E-011 | Partial column family creation (some CFs created, some not) | Upgrade from older version or partial write | Engine should detect missing CFs and create them. Versioned migration in `migrations.rs` | Medium |

### 2. Session Edge Cases

| ID | Scenario | Trigger | Expected Behavior | Priority |
|---|---|---|---|---|
| E-101 | Session with duplicate UUID | UUID collision (theoretical with v7) | Engine returns error (RocksDB put fails on same key) | Low |
| E-102 | Session with maximum metadata size | Metadata JSON > 64KB | Stored successfully (RocksDB value limit is ~3GB, but search could be slow) | Low |
| E-103 | Session listing with no filter | `list_sessions({})` | Returns ALL sessions across all projects. Should be paginated by default (limit=100) | High |
| E-104 | Session listing with offset beyond total | `list_sessions({offset: 1000})` with only 10 sessions | Returns empty list (no error) | Medium |
| E-105 | Session listing with very large limit | `list_sessions({limit: 1_000_000})` | Returns all sessions up to a hard-coded maximum (e.g., 10,000). Higher values truncated | Medium |
| E-106 | Update session with empty patch | `update_session(id, {})` | Returns the session unchanged (no-op) | Medium |
| E-107 | Update session with all fields set to None | `update_session(id, {project: null, status: null})` | Only non-null fields in the patch are applied. Null fields are ignored | Medium |
| E-108 | Create session with missing required fields | No agent_id provided | Engine returns validation error describing which fields are required | High |

### 3. Memory Edge Cases

| ID | Scenario | Trigger | Expected Behavior | Priority |
|---|---|---|---|---|
| E-201 | Memory with empty content string | `content: ""` | Created successfully. Search for empty string returns all memories? Edge case: define that empty content is allowed but only searchable via type/tag filters | Medium |
| E-202 | Memory with very large content (10MB+) | Large text body | Stored successfully in RocksDB. Search performance may degrade for keyword search on this entry (Tantivy in Phase 2 fixes this) | Medium |
| E-203 | Memory with maximum number of tags | 100+ tags | Stored successfully. Tags are an array, no hard limit. Search by tag should still work | Low |
| E-204 | Memory search with empty query | `search_memories({})` | Returns all memories up to default limit (100). Same as listing | High |
| E-205 | Memory search with special characters in keywords | `keywords: "SELECT * FROM memories; DROP TABLE"` | Treated as literal text search. No SQL injection possible (not using SQL). Keyword matching is plain text | High |
| E-206 | Memory search with Unicode content | Chinese, Arabic, emoji in content | All UTF-8 content is stored and searchable correctly | Medium |
| E-207 | Memory with session_id pointing to deleted session | Session deleted after memory creation | Memory remains orphaned. No cascade delete (by design — memories survive session deletion) | Medium |
| E-208 | Memory update changing content to empty string | `update_memory(id, {content: ""})` | Content updated to empty string. version incremented | Low |
| E-209 | Multiple memories with identical content | Same text, different session_ids | Both created successfully. Deduplication not performed at this layer | Medium |
| E-210 | Querying memories for a session that never had memories | `search_memories({session_id: <id>})` | Empty results list returned | High |

### 4. Cache Edge Cases

| ID | Scenario | Trigger | Expected Behavior | Priority |
|---|---|---|---|---|
| E-301 | Cache hit on recently written entity | Entity written via write-through | Entity is in cache. Read returns from cache without RocksDB access | High |
| E-302 | Cache miss after eviction | LRU evicts oldest entry after cache reaches capacity | Read falls through to RocksDB. Entry is re-cached | High |
| E-303 | Cache invalidation on update | Entity is updated via `update_*` | Stale entry invalidated. Next read fetches from RocksDB and re-caches | High |
| E-304 | Cache invalidation on delete | Entity is deleted | Cache entry removed. Subsequent reads return None (direct from RocksDB) | High |
| E-305 | Concurrent cache reads from 8 threads | Heavy read load | DashMap handles concurrent reads without blocking | High |
| E-306 | Cache with 0 capacity configured | `max_entries_per_type: 0` | Effectively disables caching. All reads go to RocksDB. No errors | Medium |
| E-307 | One entity type fills entire cache | 10,000 sessions, 0 memories | LRU eviction is per-type. Sessions don't evict memories. Each type has its own LRU list | High |

### 5. RocksDB Operational Edge Cases

| ID | Scenario | Trigger | Expected Behavior | Priority |
|---|---|---|---|---|
| E-401 | RocksDB write fails mid-write | Disk full during `create_session` | Error returned to caller. WAL ensures consistency — partial write doesn't corrupt DB | High |
| E-402 | RocksDB crash recovery | Process kills during `store()` | On next open, RocksDB replays WAL. Unflushed writes are recovered | High |
| E-403 | Very high write throughput to telemetry CF | Rapid telemetry events | Telemetry writes are buffered and batch-flushed (not per-event). In Phase 1, the telemetry CF exists but may not have high write volume yet | Medium |
| E-404 | Opening RocksDB with incompatible version | Version mismatch after upgrade | RocksDB handles backward-compatible format changes. Incompatible changes return error | Low |
| E-405 | Concurrent column family writes from multiple threads | StorageBackend behind Arc<RwLock> | Write lock serializes writers. Reads via read lock proceed concurrently | High |

### 6. PyO3 Bridge Edge Cases

| ID | Scenario | Trigger | Expected Behavior | Priority |
|---|---|---|---|---|
| E-501 | Python passes NaN/Inf in float field | Invalid float value | JSON deserialization fails → PyErr returned | Medium |
| E-502 | Python passes extremely large integer (>2^53) | Integer overflow in JSON | serde_json parses as Lossy or returns error. Python's JSON decoder also has this limit. Document that Python ints > 2^53 must be sent as strings | Low |
| E-503 | Python calls method with wrong argument types | `create_session("string", 123)` | PyO3 returns TypeError at the bridge boundary before Rust code executes | High |
| E-504 | Python calls method after Engine is dropped | Use-after-free | PyO3 reference counting prevents this. Engine lives as long as the Python object | Low |
| E-505 | Concurrent Python asyncio tasks calling Engine | 10 simultaneous `get_session` calls | ThreadPoolExecutor queues them. Thread pool (4 workers) processes 4 at a time. No race conditions | High |

### 7. CLI Edge Cases

| ID | Scenario | Trigger | Expected Behavior | Priority |
|---|---|---|---|---|
| E-601 | CLI run without data directory | First time user | Engine initializes with defaults. `contexter status` shows empty database with default path | High |
| E-602 | CLI session create with missing --project flag | Required argument omitted | Click validation catches missing required option and prints usage error | High |
| E-603 | CLI session get with invalid UUID format | `contexter session get not-a-uuid` | Returns error message: "Invalid UUID format" | Medium |
| E-604 | CLI pipe large output through less | `contexter session list \| less` | Output is plain text/JSON. Works normally | Low |

### 8. Compression Edge Cases

| ID | Scenario | Trigger | Expected Behavior | Priority |
|---|---|---|---|---|
| E-701 | Compress empty byte slice | `compress(b"")` | Returns empty bytes (or minimal Zstd header). LZ4 returns empty | Medium |
| E-702 | Compress already-compressed data | Double compression | Succeeds but may increase size slightly. Not an error | Low |
| E-703 | Decompress corrupted data | Bit flip in compressed buffer | Returns error with "decompression failed" message | High |
| E-704 | Decompress data compressed with different level | Level mismatch | Zstd handles any level. Same algorithm, different level — no issue | Low |
| E-705 | Very large payload for compression (100MB) | Large memory content | Zstd handles streaming. May need chunked processing. For Phase 1, small payloads expected | Medium |

---

## Error Messages

| Condition | Error Message |
|---|---|
| Data directory not writable | `"Storage path '{path}' is not writable"` |
| Database corruption detected | `"RocksDB corruption detected at {path}. Last valid checkpoint LSN: {lsn}. Run recovery from backup."` |
| Invalid UUID format | `"Invalid UUID: '{input}' is not a valid UUID v7"` |
| Entity not found for update | `"Entity of type {entity_type} with id {id} not found"` |
| Missing required field | `"Missing required field: {field_name}"` |
| RocksDB IO error | `"Storage IO error: {message}"` |
| Decompression failed | `"Decompression failed: data may be corrupted (algorithm: {algo})"` |
| Bridge panic caught | `"Internal error in storage engine: {panic_message}"` |
| CLI arg validation | `"Error: missing required option '--{flag}'"` |

---

## Recovery Paths

| Scenario | Recovery |
|---|---|
| Data directory not writable | User changes path in config.yaml or fixes permissions. Engine re-init after fix. |
| DB corruption (MANIFEST checksum failure) | Engine enters read-only mode. User is prompted to restore from backup or reinitialize. |
| Disk full during write | Error returned to caller. User frees disk space and retries. WAL ensures no partial state. |
| Concurrent write collision (same key, different data) | LWW resolved by timestamp (future CRDT). In Phase 1, last writer wins. |
| Engine init failure (any cause) | Error returned to Python layer with descriptive message. No retry — user must fix the issue and re-init. |
| WAL replay takes too long | Not implemented in Phase 1 (small WAL expected). Phase 2+ adds progress reporting. |

---

## Test Scenarios

| Scenario | Test Approach | Coverage |
|---|---|---|
| Engine init with valid path | Create temp dir, init engine, verify 8 CFs exist | E-001, E-002 |
| Engine init with invalid path | Init with read-only dir, expect error | E-005 |
| Full session CRUD lifecycle | Create → Get → List → Update → Get → Delete → Get(none) | AC-002–AC-005 |
| Full memory CRUD lifecycle | Create → Get → Search → Update → Get (version bumped) → Delete → Get(none) | AC-006–AC-010 |
| Agent + Skill CRUD | Same pattern as session | AC-011 |
| Concurrent reads (4 threads, 100 sessions) | Spawn threads, all get_session, verify no deadlock | AC-106 |
| Large content (1MB) | Create memory with 1MB content, verify round-trip | AC-107 |
| Empty database | Init fresh dir, list sessions = [], count = 0 | AC-108 |
| Cache hit/miss sequence | Read entity twice, verify counters | AC-013, AC-014 |
| PyO3 JSON round-trip | Python script imports module, creates session, gets it back | AC-017 |
| WAL checkpoint | Write N sessions, checkpoint, verify WAL size reduction | AC-019 |
| Storage size report | Write data, call storage_size(), verify per-CF non-zero | AC-020 |
| Error: invalid UUID | Pass bad UUID to create_session, expect error | AC-101 |
| Error: delete nonexistent | Delete nonexistent UUID, expect Ok | AC-103 |
| Error: update nonexistent | Update nonexistent UUID, expect error | AC-104 |
| Compression round-trip | Zstd compress + decompress, LZ4 compress + decompress | AC-018 |
| Compress empty input | Compress empty bytes, verify output | E-701 |
| Decompress corrupted data | Corrupt compressed bytes, expect error | E-703 |
| CLI session CRUD end-to-end | Run CLI commands, verify output | AC-016 |
| CLI status output | Init with data, run `contexter status`, verify sections | AC-015 |

---

## Prioritized Risk Matrix

| Risk | Likelihood | Impact | Priority |
|---|---|---|---|
| RocksDB corruption on crash | Low | High (data loss) | Critical |
| Write contention under concurrent load | Low | Medium (slowdown) | High |
| UUID collision (v7) | Negligible | Medium | Low |
| Cache eviction thrashing under mixed access | Medium | Low (hits L2) | Medium |
| PyO3 GIL deadlock | Low | High (hang) | Critical |
| CLI user provides invalid input | High | Low (error message) | Medium |
| Large memory content causes slow keyword search | Medium | Medium (Phase 2 fix) | Medium |
