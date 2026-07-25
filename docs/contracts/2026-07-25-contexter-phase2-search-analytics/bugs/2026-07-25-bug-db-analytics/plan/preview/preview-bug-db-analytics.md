# Design Preview — Bug-DB-Analytics

## Fix Plan
1. Add `Value::to_duckdb_param()` conversion function in `analytics/duckdb.rs`
2. Modify `DuckDbEngine::query()` to convert `&[Value]` into `&[&dyn ToSql]` and pass to `stmt.query()`
3. In `Engine::with_config()` (engine/mod.rs), after creating `DuckDbEngine`, call `engine.set_storage_backend(Box::new(self.storage.clone()))` 
4. In `DuckDbEngine::sync()`, use `storage_backend` to iterate column family entries via `cf.iter_forward()`
5. For each entry, parse the key/value and insert into the appropriate DuckDB table
