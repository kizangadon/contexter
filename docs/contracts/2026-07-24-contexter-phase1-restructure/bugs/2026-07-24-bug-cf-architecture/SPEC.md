# Bug 13: Column Family Architecture

## REQ-CFA-001: Separate settings into own CF
Currently, settings are stored in `CF_SESSIONS` column family (rocksdb.rs:1119-1142). Move them to a new dedicated column family. This means:
- Add `CF_SETTINGS` constant in column_families.rs
- Add it to `ColumnFamilyMap`
- Add the CF descriptor in `open_with_config` with appropriate compression
- Update `rocksdb.rs` `get_setting`/`set_setting` to use `CF_SETTINGS`
- Update `settings.rs` to reference the new CF

## REQ-CFA-002: Separate audit log into own CF
Currently, audit entries are stored in `CF_SESSIONS` (rocksdb.rs:1164). Move them to a new dedicated column family:
- Add `CF_AUDIT` constant in column_families.rs
- Add it to `ColumnFamilyMap`
- Add the CF descriptor
- Update `rocksdb.rs` to use `CF_AUDIT` for audit entries
- Update `settings.rs` `query_audit` to use the correct CF

## REQ-CFA-003: Add secondary index for session list/count
Currently, `list_sessions` and `count_sessions` do full O(n) scans of CF_SESSIONS. Add secondary indexes for sessions (project, agent_id, status) in the existing CF_MEMORY_INDEX (or a new CF_SESSION_INDEX) to support efficient filtered queries.

**Simplified approach**: Since sessions are less numerous than memories, add a single secondary index CF that stores `idx:session:<project>:<agent_id>:<status>:<uuid>` for efficient prefix-based filtering.
