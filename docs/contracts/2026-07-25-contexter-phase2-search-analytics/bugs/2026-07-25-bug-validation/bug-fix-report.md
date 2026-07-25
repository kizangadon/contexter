# Bug Fix Report — Missing EngineConfig Dimension Guard

**Bug:** `Engine::with_config()` silently accepts `vector_dimension = 0` when `enable_vector_index = true`, leading to a zero-capacity HNSW index.

**Severity:** HIGH
**Fix branch:** `feature/contexter-phase2-search-analytics`

---

## Changes Made

### 1. `contexter-core/src/error/mod.rs` — Added `InvalidConfig(String)` variant

- Added `InvalidConfig(String)` variant to `EngineError` enum with `#[error("Invalid configuration: {0}")]`
- Added `InvalidConfig` branch to `sanitized()` — preserves the message (config errors are safe to transmit)
- Added display unit test: `engine_error_display_invalid_config`
- Added sanitized unit test: `sanitized_invalid_config_preserves_message`

### 2. `contexter-core/src/engine/mod.rs` — Added dimension validation guard

- Added guard at the **start** of `Engine::with_config()` — before `RocksDbBackend::open()`:
  ```rust
  if config.enable_vector_index && config.vector_dimension == 0 {
      return Err(EngineError::InvalidConfig(
          "embedding_dim must be >= 1".into(),
      ));
  }
  ```
- Added unit tests:
  - `with_config_rejects_zero_dimension_when_vector_enabled` — AC-01
  - `with_config_succeeds_with_valid_dimension` — AC-02
  - `with_config_skips_guard_when_vector_disabled` — EC-01
  - `with_config_default_config_succeeds` — EC-02

---

## Verification

### Acceptance Criteria

| AC | Description | Status |
|----|------------|--------|
| AC-01 | `vector_dimension: 0, enable_vector_index: true` returns `Err` | ✅ PASS |
| AC-02 | `vector_dimension: 384, enable_vector_index: true` succeeds | ✅ PASS |
| AC-03 | Error message contains "embedding_dim must be >= 1" | ✅ PASS |
| AC-04 | All existing tests continue to pass | ✅ 305/305 |

### Edge Cases

| EC | Description | Status |
|----|------------|--------|
| EC-01 | `vector_dimension: 0, enable_vector_index: false` — no error | ✅ PASS |
| EC-02 | Default config (384, disabled) — succeeds | ✅ PASS |

### Test Results

```text
cargo test -p contexter-core
  result: ok. 305 passed; 0 failed; 0 ignored
  (was 298 before the fix — 7 new tests added, none broken)
```

---

## Files Modified

| File | Change |
|------|--------|
| `contexter-core/src/error/mod.rs` | Added `InvalidConfig(String)` variant + `sanitized()` handler + 2 unit tests |
| `contexter-core/src/engine/mod.rs` | Added dimension guard at top of `with_config()` + 4 unit tests |

No commits created.
