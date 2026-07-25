# Bug 13 Design Preview — CF Architecture

## New column families
1. `CF_SETTINGS` — dedicated CF for key-value settings (LZ4, 4MB target)
2. `CF_AUDIT` — dedicated CF for audit log entries (Zstd, 8MB target)
3. `CF_SESSION_INDEX` — secondary index for session filtered lookups (LZ4, 8MB target)

## Changes
- `column_families.rs`: Add 3 new CF constants + update ColumnFamilyMap
- `rocksdb.rs`: Add 3 new CF descriptors, update settings/audit CF targets, add session index write/read/scan
- `session.rs`: Update `list_sessions`/`count_sessions` to use secondary index
- `settings.rs`: Update CF references for settings/audit
- All tests that check CF count: update from 9 to 12
