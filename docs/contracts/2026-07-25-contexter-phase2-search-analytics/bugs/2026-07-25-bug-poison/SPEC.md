# Bug: Mutex Poisoning Recovery

**Severity:** MEDIUM  
**Root Cause:** `DuckDbEngine` uses `Mutex<Connection>` and `Engine` uses `RwLock<...>` without explicit poison recovery.

## Requirements

### REQ-FIX-001: Add poison recovery for DuckDbEngine Mutex
Wrap `conn.lock()` calls with `.unwrap_or_else(|e| e.into_inner())` to recover from poisoned state.

### REQ-FIX-002: Add poison recovery for Engine locks
Apply same pattern to all `RwLock`/`Mutex` accesses in `engine/mod.rs`.
