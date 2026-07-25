# Acceptance Criteria — Bug-Validation

- AC-01: `Engine::with_config(EngineConfig { enable_vector_index: true, vector_dimension: 0, .. })` returns `Err(...)`
- AC-02: `Engine::with_config(EngineConfig { enable_vector_index: true, vector_dimension: 384, .. })` succeeds
- AC-03: Error message contains "embedding_dim must be >= 1"
- AC-04: All existing tests continue to pass
