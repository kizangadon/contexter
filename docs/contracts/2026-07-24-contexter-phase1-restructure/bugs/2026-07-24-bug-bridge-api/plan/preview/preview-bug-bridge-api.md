# Bug 11 Design Preview — Bridge API Types

## Changes
1. `src/bridge.rs`: `store()` signature: `value: Vec<u8>` → `value: &str`
2. `src/bridge.rs`: `get()` return: `Option<Vec<u8>>` → `Option<String>`
3. `src/engine/maintenance.rs` (or where Engine::store/get lives): update signatures
4. Update all callers and test expectations
