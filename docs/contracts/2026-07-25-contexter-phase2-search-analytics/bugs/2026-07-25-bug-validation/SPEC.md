# Bug: Missing EngineConfig Validation

**Severity:** HIGH  
**Root Cause:** `Engine::with_config()` does not validate `vector_dimension >= 1`.

## Requirements

### REQ-FIX-001: Validate embedding_dim >= 1
Add a guard in `Engine::with_config()` that checks `config.vector_dimension >= 1` when `enable_vector_index = true`. Return `Err(EngineError::InvalidConfig("embedding_dim must be >= 1"))` on failure.

### REQ-FIX-002: Add InvalidConfig variant if missing
Add `InvalidConfig(String)` variant to `EngineError` if it doesn't already exist.
