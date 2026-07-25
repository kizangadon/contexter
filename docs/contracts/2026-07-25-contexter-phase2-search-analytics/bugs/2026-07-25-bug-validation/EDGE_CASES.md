# Edge Cases — Bug-Validation

- EC-01: `vector_dimension = 0` with `enable_vector_index = false` — should NOT error
- EC-02: `vector_dimension` not set (uses default 384) — should succeed
