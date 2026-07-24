# Bug 8: RocksDB Safety & Code Quality

## REQ-RSK-001: cf() returns EngineResult instead of panicking
The `cf()` method on `RocksDbBackend` (rocksdb.rs:177) currently panics when a column family is not found. This should return `EngineResult<&ColumnFamily>` instead, propagating errors upward.

## REQ-RSK-002: store_raw and write_batch call maybe_flush_wal
The `store_raw()` method (rocksdb.rs:1216) and `write_batch()` (rocksdb.rs:1230) do NOT call `maybe_flush_wal()` after writing. Every other mutating method does. Add `self.maybe_flush_wal()?;` calls.

## REQ-RSK-003: Remove ColumnFamilyMap field indirection
The `ColumnFamilyMap` struct (column_families.rs) stores static `&'static str` constants that are already available as `CF_*` constants. The `RocksDbBackend.cfs` field adds unnecessary indirection. However, changing this would be a significant refactor. Instead, add a `#[allow(dead_code)]` annotation and a documented reason, or mark the field as `#[cfg(test)]`. The simplest fix: wrap `ColumnFamilyMap` fields in a `#[doc(hidden)]` attribute and mark the struct as deliberately existing for forwards-compatibility.

**Actual fix**: Add `#[allow(dead_code)]` to the `ColumnFamilyMap` struct with a comment explaining it exists for forwards-compatibility. This resolves the nit without structural changes.
