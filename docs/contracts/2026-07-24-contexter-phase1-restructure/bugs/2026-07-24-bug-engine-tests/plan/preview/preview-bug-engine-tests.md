# Bug 16 Design Preview — Engine Tests

## Files to create
1. `tests/engine/session_test.rs` — Extract session tests (182 lines)
2. `tests/engine/memory_test.rs` — Extract memory tests (82 lines)
3. `tests/engine/agent_skill_test.rs` — Extract agent + skill tests (126 lines)
4. `tests/engine/settings_test.rs` — Extract settings + audit tests (161 lines)
5. `tests/engine/maintenance_test.rs` — Extract maintenance tests (133 lines)
6. `tests/engine/error_test.rs` — Extract error path + validation tests (227 lines)

## Files to modify
7. `engine/mod.rs` — Remove inline `#[cfg(test)] mod tests { ... }` block (lines 208-1205)
