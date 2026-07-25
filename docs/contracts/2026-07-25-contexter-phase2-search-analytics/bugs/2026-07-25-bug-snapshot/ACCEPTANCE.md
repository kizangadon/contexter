# Acceptance Criteria — Bug-Snapshot

- AC-01: `HnswVectorIndex::save(path)` serializes graph state to disk at path
- AC-02: `HnswVectorIndex::load_or_new(path)` loads a previously saved snapshot
- AC-03: Loaded index returns correct search results (round-trip test)
- AC-04: `Engine::shutdown()` triggers save on vector index
- AC-05: All existing tests continue to pass
