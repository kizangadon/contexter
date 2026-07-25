---
title: Phase 2 — Edge Cases Catalog
version: 1.0
date_created: 2026-07-25
---

# Edge Cases — Phase 2: Search & Analytics Engine

## Feature Overview

Phase 2 adds three optional storage tiers to Contexter. All edge cases assume the tier is enabled unless specified.

---

## Edge Case Categories

### L3: HNSW Vector Index

| ID | Scenario | Input/State | Expected Behavior | Severity | Likelihood |
|---|---|---|---|---|---|
| EC-VEC-01 | Empty index search | Index has 0 embeddings, query any vector | Return empty Vec, no panic | Low | Medium |
| EC-VEC-02 | Single-element index | Index has 1 embedding, search with k=5 | Return 1 result | Low | Medium |
| EC-VEC-03 | k larger than index size | Index has 50 embeddings, search k=1000 | Return all 50 results | Low | Medium |
| EC-VEC-04 | k=0 search | search(query, 0) | Return empty Vec | Low | Low |
| EC-VEC-05 | Dimension mismatch on insert | Index dim=384, insert with dim=128 | Return Err(DimensionMismatch) | High | Medium |
| EC-VEC-06 | Dimension mismatch on search | Index dim=384, query vec dim=768 | Return Err(DimensionMismatch) | High | Medium |
| EC-VEC-07 | Remove existing ID | Remove a valid ID from non-empty index | Succeed, len decrements by 1 | Medium | High |
| EC-VEC-08 | Remove nonexistent ID | Remove("ghost") on index with 100 entries | Succeed silently, len unchanged | Low | Medium |
| EC-VEC-09 | Remove from empty index | Remove on index with 0 entries | Succeed silently | Low | Low |
| EC-VEC-10 | Save snapshot to readonly path | save_snapshot("/proc/idx.bin") | Return IO error | Medium | Low |
| EC-VEC-11 | Load from nonexistent path | load_snapshot("/tmp/no_exist.bin") | Return IO error | Medium | Low |
| EC-VEC-12 | Load corrupt snapshot (truncated) | file has only header, no adjacency data | Return Err("corrupt snapshot") | High | Low |
| EC-VEC-13 | Load snapshot with wrong magic number | file starts with garbage bytes | Return Err("invalid magic") | High | Low |
| EC-VEC-14 | Load snapshot version mismatch | snapshot v2, code expects v1 | Return Err("version mismatch") | Medium | Low |
| EC-VEC-15 | Auto-snapshot at 1,000 mutations | Insert 1,000 embeddings sequentially | Snapshot file written with correct content | Medium | High |
| EC-VEC-16 | Multiple insert same ID | insert("a", ...) twice | Insert silently replaces (update semantics) | Medium | Medium |
| EC-VEC-17 | All-zero query vector | query = [0.0; 384] | Search returns results (cosine sim is 0 for all; order arbitrary) | Low | Medium |
| EC-VEC-18 | NaN/Inf in embedding vector | query contains NaN | Return Err("vector contains NaN/Inf") | High | Low |

### L4: Tantivy Full-Text Search

| ID | Scenario | Input/State | Expected Behavior | Severity | Likelihood |
|---|---|---|---|---|---|
| EC-FTS-01 | Empty index search | Index with 0 documents | Return empty Vec | Low | Medium |
| EC-FTS-02 | No match search | Query term that does not exist in any doc | Return empty Vec | Low | High |
| EC-FTS-03 | Special characters in query | Query contains `+ - && || ! ( ) { } [ ] ^ " ~ * ? : \` | Parse as valid Tantivy query, return subset or empty | Medium | Low |
| EC-FTS-04 | Delete nonexistent doc ID | delete("ghost") | Succeed silently | Low | Medium |
| EC-FTS-05 | Delete already-deleted doc | index("doc1"), delete("doc1"), delete("doc1") | Second delete succeeds silently | Low | Medium |
| EC-FTS-06 | Index with empty content | fields where content is "" | Index succeeds, search returns doc | Low | Medium |
| EC-FTS-07 | Very long query string | Query of 10,000 characters | Index handles gracefully (truncate or succeed) | Low | Low |
| EC-FTS-08 | Index directory read-only | Index directory without write permissions | init returns Err | High | Low |
| EC-FTS-09 | Index directory nonexistent | Path does not exist | Create directory and continue | Medium | High |
| EC-FTS-10 | Concurrent index + search | Index while another thread searches | Both succeed (Tantivy handles via Arc<Mutex<>> or RwLock<>) | Medium | Medium |
| EC-FTS-11 | Field with very long content | content = "A" * 100_000 | Index succeeds, search still works | Low | Medium |
| EC-FTS-12 | Flush on idle index | Call flush with no pending documents | Succeeds silently | Low | Medium |

### L5: DuckDB Analytics

| ID | Scenario | Input/State | Expected Behavior | Severity | Likelihood |
|---|---|---|---|---|---|
| EC-ANA-01 | Query on unsynced table | CALL query("SELECT * FROM memories") without sync first | Return Err("table memories does not exist") | High | High |
| EC-ANA-02 | Sync empty column family | Column family exists but has 0 entries | Succeed, table is empty | Low | Medium |
| EC-ANA-03 | Sync nonexistent column family | sync("nonexistent_cf") | Return Err | Medium | Low |
| EC-ANA-04 | Invalid SQL query | query("DROP TABLE memories") | Return Err (read-only or parse error) | High | Low |
| EC-ANA-05 | SQL injection attempt | query with "'; DROP TABLE;--" | DuckDB treats as string literal; no error (params are bound) | Low | Low |
| EC-ANA-06 | Double sync | sync("telemetry") twice | Second sync overwrites; data consistent | Low | Medium |
| EC-ANA-07 | Concurrent sync + query | sync in one thread, query in another | Synchronized access; no race (need RwLock or Mutex) | Medium | Low |
| EC-ANA-08 | Sync after data deleted from RocksDB | 100 records, delete 50, then sync | Table has 50 rows (current L2 state) | Medium | Medium |
| EC-ANA-09 | Analytics with no sessions | No session data in RocksDB | Efficiency query returns empty set | Low | Medium |
| EC-ANA-10 | Efficiency with zero total memories | Session with 0 total, 0 useful | Score = 0.0 (avoid division by zero) | High | Medium |

### Hybrid Search

| ID | Scenario | Input/State | Expected Behavior | Severity | Likelihood |
|---|---|---|---|---|---|
| EC-HYB-01 | L3 disabled, L4 enabled | hybrid_search with config L3=off L4=on | Return only L4 results | Medium | High |
| EC-HYB-02 | L3 enabled, L4 disabled | hybrid_search with config L3=on L4=off | Return only L3 results | Medium | High |
| EC-HYB-03 | Both tiers have no matches | No vector or text matches query | Return empty Vec | Low | Medium |
| EC-HYB-04 | RRF with k=0 | RRF parameter k=0 | Use k=1 as minimum to avoid division by zero | Medium | Low |
| EC-HYB-05 | RRF with extreme weight | weight=100.0 vector, -99.0 text | Clamp weights to [0.0, 1.0] and normalise | Medium | Low |
| EC-HYB-06 | Same ID in both result sets | Document matches both vector and FTS | Deduplicated — same ID appears once | High | High |
| EC-HYB-07 | Empty query text, valid vector | hybrid_search("", query_vec) | Rely fully on L3 score | Low | Medium |
| EC-HYB-08 | Both query text and vector are empty/invalid | hybrid_search("", &[]) | Return Err | Medium | Low |

### Engine Integration

| ID | Scenario | Input/State | Expected Behavior | Severity | Likelihood |
|---|---|---|---|---|---|
| EC-ENG-01 | All tiers disabled (default) | Default EngineConfig | Engine opens, search_memories/insert work without L3/L4/L5 | High | High |
| EC-ENG-02 | Engine opens, then lazy-enables L3 | Opening with L3 disabled, then setting config | Not supported — L3 requires restart | Low | Low |
| EC-ENG-03 | Invalid embedding dimension in config | embedding_dim = 0 | Return Err in Engine::new | High | Low |
| EC-ENG-04 | Negative embedding dimension | embedding_dim = -1 | Return Err in Engine::new | High | Low |
| EC-ENG-05 | Engine::run_analytics() with L5 disabled | L5 not configured | Return Err("analytics engine not configured") | Medium | Medium |

### Efficiency & Correlation

| ID | Scenario | Input/State | Expected Behavior | Severity | Likelihood |
|---|---|---|---|---|---|
| EC-EFF-01 | Single session efficiency | One session with 5/10 useful | efficiency = 0.5 | Low | High |
| EC-EFF-02 | Zero useful, zero total | New session with no memories | efficiency = 0.0 (zero division guard) | High | Medium |
| EC-EFF-03 | All useful, zero total useless | 100 useful, 0 total (impossible state) | Guard — total = useful + useless; assert total >= useful | High | Low |
| EC-EFF-04 | Correlation with a single session | Only 1 session in data | correlation = 0.0 (single point has no variance) | Medium | Low |
| EC-EFF-05 | Correlation with identical data | All sessions have same duration and count | correlation = 0.0 (no variance) | Low | Medium |
| EC-EFF-06 | Corrupted telemetry (negative duration) | Duration < 0 | Skip or treat as 0; return err | Medium | Low |

### Configuration & File System

| ID | Scenario | Input/State | Expected Behavior | Severity | Likelihood |
|---|---|---|---|---|---|
| EC-CFG-01 | Tantivy index path not specified | tantivy_path = None in config | L4 disabled | Low | Medium |
| EC-CFG-02 | HNSW snapshot path not specified | snapshot_path = None in config | In-memory only mode (no persistence) | Low | Medium |
| EC-CFG-03 | Custom paths for tiers | Different base dir for L3/L4/L5 | Each tier uses its own path | Low | Low |
| EC-CFG-04 | Home directory not writable | ~/.contexter/ not writable | Return Err on snapshot persist or index create | High | Low |

---

## Error Messages Reference

| Error Condition | Error Type | Suggested Message |
|---|---|---|
| HNSW dimension mismatch | `VectorError::DimensionMismatch` | "Vector dimension 128 does not match index dimension 384" |
| HNSW corrupt magic | `VectorError::InvalidMagic` | "Snapshot has invalid magic number: expected 0x484E5357, got 0xDEADBEEF" |
| HNSW version mismatch | `VectorError::VersionMismatch` | "Snapshot version 3 is not supported (max: 2)" |
| HNSW snapshot IO error | `VectorError::Io` | "Failed to save snapshot to /path: Permission denied" |
| Tantivy init path error | `FtsError::Io` | "Failed to create index directory /path: Read-only file system" |
| DuckDB table not synced | `AnalyticsError::TableNotFound` | "Table 'memories' does not exist. Call sync('memories') first" |
| Engine L5 not configured | `EngineError::AnalyticsNotConfigured` | "Analytics engine is not configured. Enable it in EngineConfig" |
| Invalid embedding dimension | `EngineError::InvalidConfig` | "embedding_dim must be >= 1, got 0" |

---

## Recovery Paths

| Failure | Recovery Action | User Visibility |
|---|---|---|
| Corrupt HNSW snapshot | Delete snapshot file, rebuild from L2 on next startup | Warning log; index rebuilt |
| Tantivy directory unwritable | Disable L4, log warning, continue without FTS | Info log at startup |
| DuckDB query on unsynced data | Call sync() first, then retry query | Auto-sync on query error |
| RRF weight sum ~= 0 | Normalise to equal weights [0.5, 0.5] | Debug log of weight override |
| Home directory nonexistent | Create ~/.contexter/ directory on engine init | Info log of directory creation |

---

## Test Scenarios Map

| Test Scenario | Edge Case IDs Covered |
|---|---|
| Vector index boundary operations | EC-VEC-01 through EC-VEC-09 |
| Snapshot corruption | EC-VEC-10 through EC-VEC-14 |
| Vector insert with bad data | EC-VEC-05, EC-VEC-06, EC-VEC-18 |
| FTS empty and edge operations | EC-FTS-01 through EC-FTS-05 |
| FTS special inputs | EC-FTS-03, EC-FTS-06, EC-FTS-07, EC-FTS-11 |
| Analytics sync lifecycle | EC-ANA-01 through EC-ANA-09 |
| Efficiency zero-division | EC-ANA-10, EC-EFF-02 |
| Hybrid search fallback modes | EC-HYB-01 through EC-HYB-04 |
| Hybrid deduplication | EC-HYB-06 |
| Engine graceful degradation | EC-ENG-01, EC-ENG-05 |
| Correlation edge cases | EC-EFF-04, EC-EFF-05 |
