# Task 3 — Remove `set_storage_backend` from trait

**Files:** `contexter-core/src/analytics/mod.rs`, `contexter-core/src/analytics/duckdb.rs`, `contexter-core/src/engine/mod.rs`

## Changes

### 1. `analytics/mod.rs` — Remove `set_storage_backend` from trait
Remove `fn set_storage_backend(&self, backend: Box<dyn Any + Send>);` from the `AnalyticsEngine` trait.

### 2. `analytics/duckdb.rs` — Keep as inherent `pub fn`
`DuckDbEngine` already implements `set_storage_backend()` inside the `impl AnalyticsEngine for DuckDbEngine` block (line 787). You need to:

1. Remove the `set_storage_backend` method from the `impl AnalyticsEngine for DuckDbEngine` block
2. Add it as an inherent `pub fn` on `DuckDbEngine` in its own `impl DuckDbEngine` block (there are already multiple `impl DuckDbEngine` blocks, so add it to a new one or an existing one)

Example:
```rust
impl DuckDbEngine {
    /// Attach a storage backend so the engine can pull data for syncing.
    pub fn set_storage_backend(&self, backend: Box<dyn Any + Send>) {
        *self.storage_backend.lock().unwrap_or_else(|e| e.into_inner()) = Some(backend);
    }
}
```

Keep the test `test_set_storage_backend` in the duckdb tests — it calls the method directly on `DuckDbEngine`, which still works since it's now an inherent method.

### 3. `engine/mod.rs` — Update call site (line 350)
Currently line 350 calls `engine.set_storage_backend(...)` where `engine` is a `DuckDbEngine` stored as `Arc<dyn AnalyticsEngine>`.

You need to change this. Since `set_storage_backend` is no longer on the trait, you can't call it through `Arc<dyn AnalyticsEngine>`. 

The local variable `engine` on line 347 is already a `DuckDbEngine` (concrete type), wrapped in `Arc::new(engine) as Arc<dyn AnalyticsEngine>` on line 351. You need to call `set_storage_backend` BEFORE wrapping it in `Arc<dyn AnalyticsEngine>`.

Change from:
```rust
let engine = crate::analytics::DuckDbEngine::new(...)?;
engine.set_storage_backend(Box::new(storage.clone())); // current line 350 — this works because engine is DuckDbEngine
Some(Arc::new(engine) as Arc<dyn AnalyticsEngine>)
```

Hmm wait, looking at the code more carefully:

```rust
let analytics_engine = if config.enable_analytics {
    let engine = crate::analytics::DuckDbEngine::new(config.analytics_cache_ttl_secs)
        .map_err(|e| EngineError::Internal(format!("Analytics init: {e}")))?;
    // Wire the storage backend so sync() can iterate real RocksDB data.
    engine.set_storage_backend(Box::new(storage.clone()));
    Some(Arc::new(engine) as Arc<dyn AnalyticsEngine>)
} else {
    None
};
```

Wait, the issue is that on line 347, `engine` is already a `DuckDbEngine` (the concrete type from `DuckDbEngine::new()`). So calling `engine.set_storage_backend(...)` works because `engine` is `DuckDbEngine`, not `Arc<dyn AnalyticsEngine>`.

So actually, no changes are needed to `engine/mod.rs` — the variable `engine` on line 347 is already a concrete `DuckDbEngine`. The call `engine.set_storage_backend(...)` on line 350 still works because it's calling the inherent method.

Let me re-read the actual engine/mod.rs code:

Lines 346-354:
```rust
let analytics_engine = if config.enable_analytics {
    let engine = crate::analytics::DuckDbEngine::new(config.analytics_cache_ttl_secs)
        .map_err(|e| EngineError::Internal(format!("Analytics init: {e}")))?;
    // Wire the storage backend so sync() can iterate real RocksDB data.
    engine.set_storage_backend(Box::new(storage.clone()));
    Some(Arc::new(engine) as Arc<dyn AnalyticsEngine>)
} else {
    None
};
```

Yes, `engine` is `DuckDbEngine` (concrete type from `DuckDbEngine::new()`). When we call `engine.set_storage_backend(...)`, it's a method call on `DuckDbEngine`. This works whether `set_storage_backend` is on the trait or inherent.

So the only change needed is:
1. `analytics/mod.rs` — remove from trait
2. `analytics/duckdb.rs` — move from trait impl block to inherent impl block

The `engine/mod.rs` call site needs NO changes.

### Update test in duckdb.rs
The test `test_set_storage_backend` calls `engine.set_storage_backend(...)` directly on `DuckDbEngine`, which still works as an inherent method.

## Verification
```bash
cargo build --workspace && cargo test --workspace
```
