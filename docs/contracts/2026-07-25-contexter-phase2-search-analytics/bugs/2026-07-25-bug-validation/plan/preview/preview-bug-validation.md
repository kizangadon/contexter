# Design Preview — Bug-Validation

## Fix Plan
In `engine/mod.rs`, add ~5 lines after `with_config()` method signature:
```rust
if config.enable_vector_index && config.vector_dimension == 0 {
    return Err(EngineError::InvalidConfig(
        "embedding_dim must be >= 1, got 0".into()
    ));
}
```
