# Acceptance Criteria — Contexter Phase 1: Rust Core Foundation

> Feature slice: Rust core engine with RocksDB storage, PyO3 bridge, and CLI diagnostics.

---

## Happy Path

### AC-001: RocksDB engine initializes with all column families

**Given** a valid data directory path  
**When** the RocksDB storage backend is created  
**Then** it opens successfully with all 8 column families (memory_items, sessions, agents, skills, efficiency_map, telemetry, conflicts, index_state)  
**And** each CF has the correct compression setting per the architecture spec

### AC-002: Session is created and retrieved

**Given** a new session payload (project, agent_id, status, metadata)  
**When** `Engine::create_session(data)` is called  
**Then** a session object is returned with a UUID v7 id, created_at, and last_active set to the current UTC time  
**And** `Engine::get_session(id)` returns the identical session  
**And** the session is retrievable after engine restart (persistence)

### AC-003: Session is listed with project filter

**Given** 5 sessions exist across 2 projects ("alpha" and "beta")  
**When** `Engine::list_sessions({project: "alpha"})` is called  
**Then** only sessions belonging to "alpha" are returned  
**And** pagination (`limit`, `offset`) correctly pages through results

### AC-004: Session is updated and changes are persisted

**Given** an existing session  
**When** `Engine::update_session(id, {status: "completed", turn_count: 5})` is called  
**Then** the returned session has the updated fields  
**And** `Engine::get_session(id)` returns the updated session

### AC-005: Session is deleted

**Given** an existing session  
**When** `Engine::delete_session(id)` is called  
**Then** subsequent `Engine::get_session(id)` returns `None`

### AC-006: Memory is created with type and tags

**Given** a valid memory payload (session_id, agent_id, type, content, tags)  
**When** `Engine::create_memory(data)` is called  
**Then** a memory object is returned with a UUID v7 id, version=1, created_at and updated_at set  
**And** `Engine::get_memory(id)` returns the identical memory

### AC-007: Memory is searched by keyword

**Given** 3 memories with different content (one containing "authentication timeout")  
**When** `Engine::search_memories({keywords: "auth"})` is called  
**Then** the memory containing "authentication timeout" is returned in the results  
**And** results are correctly ordered by relevance

### AC-008: Memory is searched with type and tag filters

**Given** memories of types "fact", "preference", and "episode", some tagged "security"  
**When** `search_memories({memory_type: "fact", tags: ["security"]})` is called  
**Then** only facts tagged "security" are returned

### AC-009: Memory version increments on update

**Given** an existing memory with version=3  
**When** `Engine::update_memory(id, {content: "new text"})` is called  
**Then** the returned memory has version=4 and updated_at > created_at

### AC-010: Memory is deleted

**Given** an existing memory  
**When** `Engine::delete_memory(id)` is called  
**Then** subsequent `Engine::get_memory(id)` returns `None`

### AC-011: Agent and skill CRUD

**Given** valid agent and skill payloads  
**When** `create_agent` and `create_skill` are called  
**Then** objects are returned with correct ids  
**And** `get_agent`, `list_agents`, `update_agent`, `delete_agent` all behave correctly  
**And** the same for skills

### AC-012: Generic key-value store works cross-CF

**Given** a key "cfg:test" and value "hello"  
**When** `store("sessions", "cfg:test", "hello")` is called  
**Then** `get("sessions", "cfg:test")` returns "hello"  
**And** `get("memory_items", "cfg:test")` returns `None` (wrong CF isolation)

### AC-013: Cache hit returns data without RocksDB read

**Given** a session that has been recently read (cached)  
**When** `get_session(id)` is called again  
**Then** the session is returned  
**And** the cache hit counter is incremented  
**And** no RocksDB read occurs (verified via telemetry or mock)

### AC-014: Cache miss falls through to RocksDB

**Given** a session that has never been read (not cached)  
**When** `get_session(id)` is called  
**Then** the session is returned  
**And** the cache miss counter is incremented  
**And** the cache is now populated (subsequent reads are hits)

### AC-015: CLI status shows correct diagnostics

**Given** a populated data directory with sessions and memories  
**When** `contexter status` is run  
**Then** output includes data directory path, per-CF sizes in bytes, total session count, total memory count, and cache hit ratio

### AC-016: CLI session CRUD

**Given** the CLI is installed  
**When** `contexter session create --project test --agent-id <uuid>` is run  
**Then** a session is created and its id is printed  
**And** `contexter session get <id>` returns the session details in JSON  
**And** `contexter session list --project test` lists the session

### AC-017: PyO3 bridge round-trips session creation

**Given** a Python script importing `contexter_core`  
**When** the Python Engine wrapper's `create_session(data)` is awaited  
**Then** a session dict is returned with camelCase keys  
**And** `get_session(id)` round-trips the same data

### AC-018: Compression round-trips correctly

**Given** a byte payload of at least 1KB of text  
**When** compressed with Zstd (level 3) and decompressed  
**Then** the decompressed output matches the original input exactly  
**And** compressed size is smaller than original  
**And** LZ4 round-trips also match

### AC-019: WAL checkpoint flushes and reduces WAL size

**Given** 100 session writes performed without explicit checkpoint  
**When** `Engine::checkpoint()` is called  
**Then** the function returns a valid LSN  
**And** WAL file size is reduced after checkpoint

### AC-020: Storage size reports per-CF breakdown

**Given** a database with at least 1 session and 1 memory  
**When** `Engine::storage_size()` is called  
**Then** it returns a dict with `per_cf` object containing entries for each CF with non-zero sizes, `wal_size` with current WAL size, and `total` as the sum

---

## Error & Edge Cases

### AC-101: Creating session with invalid UUID returns error

**Given** an agent_id that is not a valid UUID  
**When** `Engine::create_session({agent_id: "not-a-uuid"})` is called  
**Then** a `PyErr` (or Rust `Err`) is returned with a message indicating invalid UUID format

### AC-102: Getting non-existent entity returns None

**Given** a UUID that does not correspond to any session  
**When** `Engine::get_session(nonexistent_id)` is called  
**Then** `None` is returned (not an error)

### AC-103: Deleting non-existent entity returns Ok

**Given** a UUID that does not correspond to any session  
**When** `Engine::delete_session(nonexistent_id)` is called  
**Then** `Ok(())` is returned (idempotent delete)

### AC-104: Updating non-existent entity returns error

**Given** a UUID that does not correspond to any session  
**When** `Engine::update_session(nonexistent_id, {status: "completed"})` is called  
**Then** an error is returned indicating entity not found

### AC-105: Storage path not writable returns error on init

**Given** a storage path that points to a read-only directory  
**When** the Engine is initialized with that path  
**Then** an error is returned indicating the path is not writable

### AC-106: Concurrent reads from multiple threads succeed

**Given** an engine with 100 stored sessions  
**When** 4 threads simultaneously call `get_session` for different ids  
**Then** all calls return the correct sessions  
**And** no deadlocks or panics occur

### AC-107: Large content (1MB) memory can be stored and retrieved

**Given** a memory with content of 1MB  
**When** `Engine::create_memory(data)` is called  
**Then** the memory is created successfully  
**And** `get_memory` retrieves the full 1MB content correctly

### AC-108: Engine works with empty database

**Given** a fresh data directory with no existing data  
**When** the Engine is initialized  
**Then** no errors occur  
**And** `list_sessions({})` returns an empty list  
**And** `count_sessions({})` returns 0

---

## Performance & Non-Functional

### AC-201: Cache read latency under 100µs

**Given** a hot cache with entries  
**When** `get_session(id)` is called for a cached session  
**Then** the call completes in under 100 microseconds (measured programmatically)

### AC-202: RocksDB write latency under 5ms

**Given** the engine is running  
**When** `Engine::create_session(data)` is called  
**Then** the call completes in under 5 milliseconds under normal conditions

### AC-203: All tests pass

**Given** the Rust codebase  
**When** `cargo test` is executed  
**Then** all tests pass  
**And** `cargo clippy -- -D warnings` produces no warnings

### AC-204: Test coverage meets threshold

**Given** the Rust codebase  
**When** coverage is measured  
**Then** all public functions have at least one test  
**And** all error paths are tested
