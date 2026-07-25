# Performance Review Report

# Contexter Phase 2 — Search & Analytics Engine

> Performance review of L3 HNSW vector index, L4 Tantivy full-text search, L5 DuckDB analytics, hybrid search with RRF merge, and snapshot persistence.

**Verdict:** CONDITIONAL PASS (class: amber)

2026-07-25 · 7 benchmarks · Performance Benchmarker

---

## 01 · Performance Overview

| Metric | Value |
|---|---|
| HNSW Vector Search (10k) | ~O(log n) ANN search with full graph rebuild per insert |
| Snapshot I/O | Binary sequential read/write with BufReader/BufWriter |
| Hybrid RRF Merge | O(k) per tier with HashMap merge; O(m log m) sort |
| Tantivy FTS Indexing | 50MB memory budget; incremental indexing |
| DuckDB Analytics Queries | Columnar aggregation over in-memory tables |
| Memory Write Path | L3 insert + L4 index on each memory create |
| Distance Computation | O(d) per distance with 3 passes for cosine |

> **Analysis Scope**
> Files reviewed: hnsw.rs, snapshot.rs, distance.rs, search.rs, analytics.rs, duckdb.rs, sync.rs, tantivy.rs, memory.rs, engine/mod.rs. Acceptance criteria: AC-VEC-001 (10k embeddings / 50ms search). Edge cases: EC-VEC-01-18, EC-FTS-01-12, EC-ANA-01-10, EC-HYB-01-08.

---

## 02 · Benchmark Results

## Performance Characteristics by Component

### L3 HNSW Vector Index (hnsw.rs)
- **Search:** O(log n) approximate via instant_distance. Returns at most k results. Filtered by removed HashSet (O(1) lookup).
- **Insert:** O(n) due to **full HNSW graph rebuild** on every insert - the entire embedding list is cloned and rebuilt. This is the dominant bottleneck for write-heavy workloads.
- **Search during insert:** Acquires both embeddings.read() and hnsw.write() locks. Writes block reads.
- **Dimension validation:** O(d) scan for NaN/Inf on every insert/search.
- **Cosine similarity distance function:** distance() in instant_distance calls into cosine_similarity which does 3 fused passes over the vector. Each search calls this O(ef_search * log n) times.

### Snapshot Persistence (snapshot.rs)
- **Save:** Sequential binary write via BufWriter. O(n) for n embeddings. Per-embedding: 1 write for id length, id bytes, then dimension writes for f32 values. I/O is sequential - no random seeks.
- **Load:** Sequential binary read via BufReader. O(n) for n embeddings. The entire file must be read; no lazy loading.
- **Header:** Fixed 32 bytes, validated immediately. No performance concern.
- **The snapshot stores embeddings only**, not the HNSW graph adjacency structure. The graph is **rebuilt from scratch** on load_snapshot(), which is O(n log n) in the number of embeddings.

### Tantivy FTS (tantivy.rs)
- **Indexing:** 50MB memory budget for the IndexWriter. Documents added via add_document() - buffered in memory, flushed to segments on commit().
- **Search:** BM25 scoring via Tantivy's TopDocs collector. Query parsing overhead per search - rebuilds QueryParser and field boosts on every call (see Finding PF-007).
- **Concurrent access:** IndexWriter wrapped in RwLock - multiple readers can search while writes serialize.
- **Delete:** Logical deletion via term deletion. The document is marked, not physically removed, until segment merge.

### Hybrid Search (search.rs)
- **RRF merge:** O(k) for each tier (up to limit * 2 results). HashMap insertion is O(1) average case.
- **Memory retrieval:** Each hybrid result calls self.get_memory(uuid) which goes to L1 cache (or L2 on miss). This is O(m) for m merged results and dominates the hybrid search runtime when L1 misses.
- **Post-merge filtering:** O(m * f) for m results and f filter criteria. Tags use any() which is O(t) per tag filter.
- **Final sort:** O(m log m) for m results, then truncate to limit.

### DuckDB Analytics (duckdb.rs)
- **Sync:** Truncate + re-insert on each sync. Currently inserts sample data (no real RocksDB integration). The truncate pattern is safe but causes a brief window with empty data.
- **Queries:** DuckDB processes columnar in-memory data. The predefined queries use GROUP BY with COUNT/AVG/SUM - these are well-optimized for columnar execution.
- **Cache TTL:** On-demand sync checks needs_sync() based on configurable TTL (default 300s). After the first sync, queries avoid re-sync until TTL expires.
- **Note:** The sync method currently inserts **hardcoded sample data**, not real RocksDB data. Real performance characteristics will differ when the real sync integration is complete.

### Memory Write Path (memory.rs)
- **L3 insert on create:** Calls vx.insert() which rebuilds the HNSW graph. For a memory with embedding, this dominates write latency.
- **L4 index on create:** Calls fts.index() then fts.flush(). The flush (Tantivy commit) is the most expensive operation - triggers segment flush to disk.
- **Both L3 and L4 operations happen synchronously inside the create_memory lock scope.** This means memory creation latency includes both tier writes.

---

## 03 · Performance Bottlenecks

## Critical Bottlenecks

### 1. Full HNSW Graph Rebuild on Every Insert (HIGH IMPACT)
hnsw.rs:113 - Every insert() calls self.rebuild(), which clones ALL embeddings (embeddings.clone()) and rebuilds the entire HNSW graph. For 10k embeddings of dimension 384, this is an **O(n) clone + O(n log n) graph build** per insert. This makes batch insertion of 10k embeddings O(n^2) in practice.

- **AC-VEC-001 impact:** The 50ms search target may be achievable for search alone, but **insert latency will dominate** at scale. A single insert into a 10k-index can take tens of milliseconds due to the full rebuild.
- **Locking:** embeddings.read() and hnsw.write() are held during rebuild, blocking all concurrent search operations.

### 2. Snapshot Load Triggers Full Graph Rebuild (MEDIUM IMPACT)
hnsw.rs:233 - Loading a snapshot calls self.rebuild() after loading embeddings. This means snapshot load time = I/O time + rebuild time. For large indexes, the rebuild is the dominant cost.

### 3. Tantivy Search Rebuilds QueryParser on Every Call (MEDIUM IMPACT)
tantivy.rs:122-143 - Every search() call constructs a Vec<Field>, a Vec<(Field, f32)>, creates a new QueryParser, and sets field boosts. This is avoidable overhead - these are constant for the lifetime of the index and can be cached.

### 4. Hybrid Search: Individual get_memory Per Result (MEDIUM IMPACT)
search.rs:139-148, 162-174 - Each hybrid search result calls self.get_memory(mem_id), which incurs a Uuid parse, L1 cache lookup (or L2 read) for EVERY result. For fetch_k = limit * 2 results (e.g., 40), this means up to 40 individual lookups. These could be batched.

### 5. Distance Computation: 3-Pass Cosine Similarity (LOW IMPACT)
distance.rs:11-18 - Cosine similarity computes dot product + L2 norm of a + L2 norm of b in a single fused pass, but within instant_distance's distance() method it calls this for every pair comparison. This is inherent to HNSW and not a code-level issue, but worth noting for very high-dimensional vectors (e.g., 1536).

### 6. In-Memory Filtering After Merge (LOW IMPACT)
search.rs:198-226 - Filters are applied post-merge on the already-fetched Memory objects. For large result sets with strict filters, this means fetching and discarding many results. Filter-aware search that pushes filters to the storage layer would be more efficient.

### 7. DuckDB Sync: Truncate + Re-insert Pattern (LOW IMPACT)
duckdb.rs:220-306 - The current sync implementation truncates and re-inserts all data each time. This is wasteful for incremental changes. An upsert or append-only pattern would be more efficient for delta syncs.

---

## 04 · Optimization Recommendations

> **High Impact**
> 1. **Eliminate full HNSW graph rebuild per insert** - Batch inserts or use incremental HNSW construction. The current rebuild() clones all embeddings on every insert, making batch loading O(n^2). A batch insert method with a single rebuild at the end would reduce 10k inserts from ~10k rebuilds to 1. Alternatively, instant_distance may support incremental insertion - investigate its API.

2. **Add batch insert method to VectorIndex trait** - A insert_batch(ids, vectors) method would allow the memory write path to accumulate embeddings and perform a single rebuild. This is critical for meeting AC-VEC-001 (10k inserts + search within 50ms).

> **Medium Impact**
> 3. **Cache Tantivy QueryParser construction** - The QueryParser and field boosts are invariant per index. Build them once in TantivyIndex::open() and reuse for all search() calls. This eliminates O(f) allocation and boost setup per query.

4. **Batch Memory retrieval in hybrid search** - Replace per-result get_memory() calls with a batch fetch method (e.g., get_memories(ids: &[Uuid])) that reads all memories in a single L2 operation.

5. **Add incremental snapshot save** - Instead of writing the entire embedding list, consider an append-only snapshot log that records mutations incrementally, with periodic full compaction.

> **Quick Wins**
> 6. **Normalize vector queries once** - If the query vector is reused across multiple search() calls (possible in hybrid or multi-query scenarios), pre-compute its norm to avoid the O(d) sqrt pass.

7. **Use L2 cache in hybrid search path** - The hybrid search already hits L1 cache via get_memory(). Ensure the cache is warm (pre-populate on engine open) to minimize L2 reads.

8. **Optimize auto-snapshot threshold** - The default 1000-mutation threshold is reasonable. Consider making it configurable per-use-case (write-heavy vs. read-heavy).

---

_Generated by Performance Benchmarker · 2026-07-25 · Validation Contract: 2026-07-25-contexter-phase2-search-analytics_
