# Security Review Report

# Contexter Phase 1 — Core Storage Engine

> Rust storage engine (RocksDB multi-tier) with PyO3 bridge and CLI. Stores sessions, memories, agents, skills, settings, and audit logs.

**Verdict:** CONDITIONAL PASS (class: amber)

2026-07-23 · **7 findings** · Security Architect

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | **2** |
| Medium | **3** |
| Low | **2** |

> **Security Scope**
> Phase 1 is a local-first storage engine. The attack surface includes the PyO3 Python bridge (data deserialization boundary), the CLI `--db-path` argument, the LZ4 decompression path (compression bomb risk), and key/value injection vectors in the settings API. Authentication/authorization is out of scope — this layer trusts its caller entirely.

---

## 02 · Vulnerability Findings

### 🔴 SEC-001 (High) — LZ4 decompression has no output-size bound (compression bomb)

| Field | Value |
|-------|-------|
| **File** | `src/compression/mod.rs:80` |
| **Risk** | A crafted small compressed payload may decompress to gigabytes, causing OOM |
| **Attack vector** | Any path that accepts compressed data and calls `Lz4Compression::decompress` |

The `Lz4Compression::decompress` method calls:
```rust
lz4::block::decompress(data, None)
```
The `None` parameter (maximum decompressed size) means the library uses a generous default. An attacker who can inject compressed data can craft a "compression bomb" — a small input that decompresses to an extremely large buffer. This is a classic zip-bomb / decompression bomb vector.

**Mitigation**: Pass a reasonable `max_decompressed_size` (e.g., 64 MB). Or, since the `compression` feature is gated and not currently wired into the storage path (RocksDB handles its own compression at the CF level), confirm that the `Compression` trait is **only** exposed to trusted callers and document this restriction.

**Remediation**:
```rust
// In src/compression/mod.rs, line 80:
const MAX_DECOMPRESS_SIZE: usize = 64 * 1024 * 1024; // 64 MB

fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, EngineError> {
    lz4::block::decompress(data, Some(MAX_DECOMPRESS_SIZE as i32))
        .map_err(|e| EngineError::Compression(format!("lz4 decompress: {e}")))
}
```

---

### 🔴 SEC-002 (High) — No input size validation on memory content or serde_json::Value fields

| Field | Value |
|-------|-------|
| **Files** | `src/types/mod.rs`, `src/storage/rocksdb_backend.rs` |
| **Risk** | Unbounded memory content (1 MB tested), unbounded JSON depth/nesting — DoS by resource exhaustion |
| **Attack vector** | Python bridge or CLI can accept arbitrarily large `content`, `metadata`, `config`, or `changes` JSON values |

The `Memory.content` field is a `String` with no size bound. The test `test_memory_large_content` creates a 1 MB entry (line 1486). The `metadata` field in `Session`, `config` in `Agent`, and `changes` in `AuditEntry` are typed as `serde_json::Value`, which can accept arbitrarily deep nesting.

**Attack scenarios**:
1. **Memory exhaustion**: Send a multi-GB content string that exhausts RAM before RocksDB rejects it.
2. **Stack overflow from deeply nested JSON**: A deeply nested `serde_json::Value` (e.g., 10,000 levels of `{"a":{"a":...}}`) during deserialization can cause a stack overflow in serde_json's recursive parser.

**Remediation**: Add maximum size constraints at the trust boundary (Python bridge and CLI):
```rust
const MAX_CONTENT_SIZE: usize = 512 * 1024; // 512 KB
const MAX_JSON_DEPTH: usize = 32;

// Validate at create/update boundaries in python.rs and cli.rs
if content.len() > MAX_CONTENT_SIZE {
    return Err(EngineError::Validation("content exceeds maximum size".into()));
}
```

---

### 🟡 SEC-003 (Medium) — serde_json deserialization of untrusted input at Python boundary

| Field | Value |
|-------|-------|
| **File** | `src/python.rs` (all `serde_json::from_str` calls, lines 92, 114, 128, 149, 164, 184, 193, 211, 226, 247, 255, 280, 301, 309, 347, 358) |
| **Risk** | Stack overflow via deeply nested JSON; resource exhaustion via huge payloads |
| **Attack vector** | Python caller passes untrusted JSON strings to any bridge method |

Every `serde_json::from_str` call in the PyO3 bridge treats its input as untrusted. While `serde_json` is memory-safe (no buffer overflows), it is **not** resilient against:
- **Deeply nested objects** (e.g., `[[[...]]]` at depth > 512) — causes stack overflow 💥
- **Very large payloads** (e.g., 500 MB JSON array) — causes OOM

serde_json's recursive descent parser uses the call stack for nested structures. A crafted payload can cause a stack overflow that terminates the process.

**Remediation**: Use `serde_json::Deserializer` with depth limiting:
```rust
use serde_json::Deserializer;
use serde::de::DeserializeOwned;

fn deserialize_with_depth_limit<'de, T: DeserializeOwned>(json: &'de str) -> Result<T, serde_json::Error> {
    let mut deserializer = Deserializer::from_str(json);
    deserializer.set_max_depth(32); // Reject deeply nested input
    T::deserialize(&mut deserializer)
}
```

Or validate input size before deserialization at the bridge boundary.

---

### 🟡 SEC-004 (Medium) — Settings keys not validated for length or content

| Field | Value |
|-------|-------|
| **Files** | `src/storage/rocksdb_backend.rs:917-938`, `src/python.rs:330-339`, `src/cli.rs:954-968` |
| **Risk** | Oversized keys consume disproportionate storage; keys with special characters may cause confusion with key prefix scheme |
| **Attack vector** | Any caller passing a `key` to `set_setting` or `get_setting` |

The `setting_key()` function prefix the key with `cfg:`, but does not validate the key itself:
```rust
fn setting_key(key: &str) -> String {
    format!("{KEY_PREFIX_SETTING}{key}")
}
```

An attacker could:
- Pass a key of length `usize::MAX` (resource exhaustion)
- Pass a key containing null bytes or non-UTF-8 bytes (while `&str` ensures valid UTF-8, the resulting key may collide with other internal keys)
- Pass a key with the prefix `cfg:` itself, creating confusing double-prefixed keys

**Remediation**: Add validation at the boundary:
```rust
const MAX_SETTING_KEY_LENGTH: usize = 256;

if key.is_empty() || key.len() > MAX_SETTING_KEY_LENGTH {
    return Err(EngineError::Validation(format!(
        "setting key must be 1-{MAX_SETTING_KEY_LENGTH} characters"
    )));
}
```

---

### 🟡 SEC-005 (Medium) — Error messages expose entity IDs in NotFound errors

| Field | Value |
|-------|-------|
| **File** | `src/error.rs:16-22` |
| **Risk** | Information disclosure if error propagates across a trust boundary |
| **Attack vector** | Error messages returned to callers include internal entity UUIDs |

The `NotFound` error variant renders:
```
Entity not found: Session abc-123
```

If this error propagates through an HTTP API or similar service boundary in a future phase, it reveals the exact entity ID that was searched for, which could be used to enumerate valid UUIDs.

**Remediation**: Separate user-facing errors from internal diagnostic errors. For now, document that `EngineError::NotFound` should be converted to a generic `"not found"` message before crossing any network boundary.

---

### 🔵 SEC-006 (Low) — CLI `--db-path` path validation is minimal

| Field | Value |
|-------|-------|
| **File** | `src/cli.rs:36-37`, `src/storage/rocksdb_backend.rs:126-128` |
| **Risk** | Accidental DB creation in unintended locations; no symlink validation |
| **Attack vector** | User specifies a path to a sensitive directory |

The CLI accepts `--db-path` with default `./contexter_data`. RocksDB will create the directory if missing (`create_if_missing: true`). There is no:
- Canonicalization of the path
- Check that the path is not a symlink to a sensitive location
- Warning when pointing to an existing non-RocksDB directory

For a CLI tool this is acceptable risk, but worth hardening:
```rust
use std::path::Path;
let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
// Warn if directory exists but doesn't look like a RocksDB database
```

---

### 🔵 SEC-007 (Low) — Skill.file_path stored without sanitization (future risk)

| Field | Value |
|-------|-------|
| **File** | `src/types/mod.rs:333`, `src/storage/rocksdb_backend.rs:804` |
| **Risk** | Path traversal in future phases when file_path is used to read/execute skill files |
| **Attack vector** | Stored file_path could be used for path traversal if this field is dereferenced in a future phase |

The `Skill.file_path` field is an `Option<String>` that stores an arbitrary filesystem path. In Phase 1 this is just stored and returned — it is **not** used to read or execute any files. However, if a future phase loads skills from these paths, a malicious path like `../../etc/passwd` could cause a path traversal.

**Remediation**: Document this risk, and if `file_path` is ever used in a future phase, validate that it:
1. Is within an allowed directory
2. Has no `../` or absolute path components
3. Points to an expected file type

---

## 03 · Security-Critical Code Highlights

### Positive — Strong durability guarantees
Every write operation calls `flush_wal(true)` — synchronous WAL flush. This ensures data is durable to disk before returning:
- `src/storage/rocksdb_backend.rs` — lines 292, 379, 388, 452, 593, 602, 676, 778, 787, 813, 900, 909, 937, 961

### Positive — Python bridge is feature-gated
```rust
#[cfg(feature = "python")]
pub mod python;
```
The entire Python attack surface is only compiled when explicitly enabled.

### Positive — Thread safety verified
```rust
fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}
assert_send::<Engine>();
assert_sync::<Engine>();
```
Both `Engine` and `PyEngine` have compile-time trait bound verification in tests.

### Positive — UUID validation at all boundaries
Every UUID string from CLI or Python is validated via `Uuid::parse_str` before use, rejecting invalid inputs with clear error messages.

### Positive — TempDir cleanup in all tests
All test functions bind `TempDir` to `_dir` or a named variable, ensuring automatic cleanup when the test completes.

---

## 04 · Remediation Recommendations

> **Must Fix**
> - **SEC-001**: Add decompression size bound to `Lz4Compression::decompress` (or document that the `compression` feature is for trusted data only)
> - **SEC-002**: Add input size limits on `Memory.content` and JSON depth limits on `serde_json::Value` fields at the PyO3 bridge boundary

> **Should Fix**
> - **SEC-003**: Use `serde_json::Deserializer::set_max_depth()` in the Python bridge to prevent stack overflow from deeply nested JSON
> - **SEC-004**: Validate setting key length (1-256 chars) and reject empty keys at the API boundary
> - **SEC-005**: Document that `EngineError::NotFound` must be converted to a generic error before crossing network boundaries

> **Consider**
> - **SEC-006**: Add path canonicalization and existence checks to CLI `--db-path`
> - **SEC-007**: Document `Skill.file_path` as a future path traversal risk and validate if dereferenced

---

_Generated by Security Architect · 2026-07-23 · Validation Contract: 2026-07-23-contexter-phase1_
