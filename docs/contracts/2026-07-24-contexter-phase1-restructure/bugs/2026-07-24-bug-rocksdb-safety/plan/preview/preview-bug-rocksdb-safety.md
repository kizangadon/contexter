# Bug 8 Design Preview — RocksDB Safety

## Changes
1. `rocksdb.rs`: Change `cf()` signature from `&ColumnFamily` to `EngineResult<&ColumnFamily>`, update 20+ callers with `?`
2. `rocksdb.rs`: Add `self.maybe_flush_wal()?;` to `store_raw()` and `write_batch()`
3. `column_families.rs`: Add `#[allow(dead_code)]` to `ColumnFamilyMap`
