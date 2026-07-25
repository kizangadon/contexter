# Acceptance Criteria

### AC-01: Batch get_memories
GIVEN a list of N memory IDs  
WHEN `get_memories` is called on RocksDbBackend  
THEN all N memories MUST be fetched in a single operation (not N individual get calls)

### AC-02: Read queries not blocked by sync
GIVEN a sync operation is in progress  
WHEN a query like `get_efficiency_scores` is called  
THEN it MUST NOT be blocked by the sync's write lock

### AC-03: Incremental sync
GIVEN a sync operation with existing data  
WHEN it runs again  
THEN it MUST only process new/changed records, not truncate+re-insert everything
