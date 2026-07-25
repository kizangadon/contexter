# Acceptance Criteria

### AC-01: Drop calls shutdown
GIVEN an `Engine` is dropped  
WHEN the drop happens  
THEN `shutdown()` MUST be called (snapshot saved, thread joined)

### AC-02: Idempotent shutdown
GIVEN `shutdown()` has already been called  
WHEN it is called again  
THEN it MUST NOT panic and MUST NOT cause undefined behavior

### AC-03: Thread join
GIVEN the periodic snapshot thread is running  
WHEN `shutdown()` is called (via drop or directly)  
THEN the thread MUST be joined before the method returns
