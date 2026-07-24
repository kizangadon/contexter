# Bug 19: Remove Double Sync in store_raw

## REQ-DSY-001: Remove redundant WriteOptions::set_sync(true)
In `src/storage/rocksdb.rs`, the `store_raw` method (around line 1402) uses both `WriteOptions::set_sync(true)` on `put_cf_opt` AND `maybe_flush_wal()`. This causes TWO fsyncs per write call (~4-20ms I/O wait). All other write paths in the crate use only `maybe_flush_wal()`. Remove `set_sync(true)` to make `store_raw` consistent.
