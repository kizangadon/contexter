# Bug 3 — Security Hardening: Implementation Report

**Date:** 2026-07-24  
**Branch:** `feature/contexter-phase1-core`  
**Feature:** Security hardening — input validation boundaries, error sanitization, path safety  

---

## Verification Summary

| Scope | Status |
|---|---|
| `cargo test` (179 tests) | ✅ PASS — 179/179 pass |
| `cargo clippy --all-targets -- -D warnings` | ✅ CLEAN — zero warnings |
| Build | ✅ CLEAN |

---

## Fix 1: LZ4 64MB decompress limit

**File:** `src/compression/mod.rs` (lines 89–104)

```rust
fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, EngineError> {
    const MAX_DECOMPRESSED_SIZE: usize = 64 * 1024 * 1024;

    let decompressed = lz4::block::decompress(data, None)
        .map_err(|e| EngineError::Compression(format!("lz4 decompress: {e}")))?;

    if decompressed.len() > MAX_DECOMPRESSED_SIZE {
        return Err(EngineError::Compression(
            format!("Decompressed data too large: {} bytes (max {})",
                decompressed.len(), MAX_DECOMPRESSED_SIZE)
        ));
    }

    Ok(decompressed)
}
```

- **Test:** `lz4_rejects_oversized_decompression` — generates 65MB of zeros, compresses, then asserts decompression is rejected with "too large" in the error message. **PASS.**
- **Edge case:** `lz4_empty_input_decompress_returns_empty` — empty round-trip returns empty vec (not error). **PASS.**

---

## Fix 2: Zstd 128MB decompress limit

**File:** `src/compression/mod.rs` (lines 56–73)

```rust
fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, EngineError> {
    const MAX_DECOMPRESSED_SIZE: usize = 128 * 1024 * 1024;

    let decompressed = zstd::decode_all(std::io::Cursor::new(data))
        .map_err(|e| EngineError::Compression(format!("zstd decompress: {e}")))?;

    if decompressed.len() > MAX_DECOMPRESSED_SIZE {
        return Err(EngineError::Compression(
            format!("Decompressed data too large: {} bytes (max {})",
                decompressed.len(), MAX_DECOMPRESSED_SIZE)
        ));
    }

    Ok(decompressed)
}
```

- **Test:** `zstd_rejects_oversized_decompression` — generates 129MB of zeros, compresses, asserts rejection. **PASS.**
- **Edge case:** `zstd_empty_data` — empty round-trip works. **PASS.**

---

## Fix 3: Memory content > 1MB rejection

**File:** `src/engine/mod.rs` (lines 247–254)

```rust
pub fn create_memory(&self, new_memory: NewMemory) -> EngineResult<Memory> {
    self.stats.memories_created.fetch_add(1, Ordering::Relaxed);
    if new_memory.content.len() > 1024 * 1024 {
        return Err(EngineError::Validation(
            "Memory content exceeds 1MB limit".into(),
        ));
    }
    // ...
}
```

- **Test:** `test_memory_content_exactly_1mb_succeeds` — boundary test, 1MB content accepted. **PASS.**
- **Test:** `test_memory_content_exceeds_limit_rejected` — 1MB+1 byte rejected with error mentioning "1MB". **PASS.**

---

## Fix 4: Setting key validation (empty / > 256 chars)

**File:** `src/engine/mod.rs` (lines 452–463)

```rust
pub fn set_setting(&self, key: &str, value: &str) -> EngineResult<()> {
    if key.is_empty() || key.len() > 256 {
        return Err(EngineError::Validation(
            "Setting key must be 1-256 characters".into(),
        ));
    }
    // ...
}
```

- **Test:** `test_setting_empty_key_rejected` — empty key → error. **PASS.**
- **Test:** `test_setting_key_too_long_rejected` — 257 chars → error. **PASS.**
- **Test:** `test_setting_key_256_chars_succeeds` — boundary, 256 chars → OK. **PASS.**
- **Test:** `test_setting_valid_key_accepted` — normal key → OK. **PASS.**

---

## Fix 5: `EngineError::sanitized()` method

**File:** `src/error.rs` (lines 48–65)

```rust
pub fn sanitized(&self) -> String {
    match self {
        EngineError::NotFound { .. } => "Resource not found".to_string(),
        EngineError::Validation(msg) => format!("Validation error: {msg}"),
        EngineError::Storage(_) => "Storage error".to_string(),
        EngineError::Serialization(_) => "Serialization error".to_string(),
        EngineError::Compression(_) => "Compression error".to_string(),
        EngineError::Cache(_) => "Cache error".to_string(),
        EngineError::Internal(_) => "Internal error".to_string(),
    }
}
```

- `NotFound` → strips both `entity_type` and `id`. Returns `"Resource not found"`.
- `Storage`/`Serialization`/`Compression`/`Cache`/`Internal` → returns generic message, discards details.
- `Validation` → preserves the message (safe for user consumption).
- **Tests (7):** `sanitized_not_found_strips_ids`, `sanitized_validation_preserves_message`, `sanitized_storage_is_generic`, `sanitized_serialization_is_generic`, `sanitized_compression_is_generic`, `sanitized_cache_is_generic`, `sanitized_internal_is_generic` — **ALL PASS.**

---

## Fix 6: CLI path validation and /tmp warning

**File:** `src/cli.rs` (lines 508–523)

```rust
let db_path_obj = std::path::Path::new(&db_path);
if db_path_obj.exists() && !db_path_obj.is_dir() {
    eprintln!("Error: '{}' exists but is not a directory", db_path);
    std::process::exit(1);
}
let canonical = db_path_obj
    .canonicalize()
    .unwrap_or_else(|_| {
        std::path::PathBuf::from(&db_path)
    });
if canonical.starts_with("/tmp") {
    eprintln!("Warning: data in {} may be lost on reboot", canonical.display());
}
```

- Non-directory path → prints error and exits with code 1.
- `/tmp` path → prints warning about data loss on reboot.
- Non-existent path → no error (Engine::open creates it).
- **Edge case per contract:** Non-existent path should create directory, not error — satisfied since `canonicalize()` fallback to the raw path and `Engine::open` creates it.

---

## Fix 7: Doc comment on `Skill.file_path`

**File:** `src/types/mod.rs` (lines 317–326)

```rust
/// A registered capability or tool that an agent can use.
///
/// # Security note — `file_path` validation
///
/// The [`file_path`](Skill::file_path) field is an optional filesystem path
/// supplied by the caller. It is **not validated or canonicalised** before
/// storage or retrieval, which could enable path-traversal attacks if a
/// downstream consumer uses the path without sanitisation (e.g. to load or
/// execute a file). Future work should add an allow-list or canonicalisation
/// step at the API boundary.
```

The doc comment on `Skill` (which documents `file_path`) explicitly warns about path traversal risk.

---

## Fix 8: `Validation` variant on `EngineError`

**File:** `src/error.rs` (lines 24–26)

```rust
/// A validation constraint was violated.
#[error("Validation error: {0}")]
Validation(String),
```

The `Validation` variant exists with `Display` and a `String` payload. Used by fixes 3 and 4.

---

## Acceptance Criteria Verification

| AC | Description | Status |
|---|---|---|
| AC-1 | LZ4 refuses output > 64MB | ✅ |
| AC-2 | Zstd refuses output > 128MB | ✅ |
| AC-3 | `Engine::create_memory` rejects content > 1MB | ✅ |
| AC-4 | `Engine::set_setting` rejects empty key | ✅ |
| AC-5 | `Engine::set_setting` rejects key > 256 chars | ✅ |
| AC-6 | `sanitized()` returns generic messages without IDs | ✅ |
| AC-7 | CLI warns when path is in `/tmp` | ✅ |
| AC-8 | CLI rejects non-directory paths | ✅ |
| AC-9 | `Skill.file_path` has doc comment about path traversal | ✅ |
| AC-10 | `Validation` variant exists on `EngineError` | ✅ |
| AC-11 | `cargo test` passes | ✅ — 179/179 |
| AC-12 | `cargo clippy -- -D warnings` clean | ✅ |

---

## Edge Case Verification

| Edge Case | Status |
|---|---|
| LZ4 empty input → empty vec (not error) | ✅ |
| Memory content exactly 1MB → succeeds | ✅ |
| Memory content 1MB+1 byte → rejected | ✅ |
| Setting key 256 chars → succeeds | ✅ |
| Setting key 257 chars → rejected | ✅ |
| `sanitized()` on all variants → no ID leak | ✅ |
| CLI path non-existent → creates directory | ✅ |

---

## Conclusion

All 8 fixes are **fully implemented and verified**. All 12 acceptance criteria pass. All 7 edge cases are handled correctly. Build, tests (179/179), and clippy are clean.
