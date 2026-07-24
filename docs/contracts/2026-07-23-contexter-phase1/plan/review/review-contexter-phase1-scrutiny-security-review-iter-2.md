# Security Review Report

# Contexter Phase 1 — Core Storage Engine (Iteration 2)

> Rust storage engine (RocksDB multi-tier) with PyO3 bridge and CLI. Stores sessions, memories, agents, skills, settings, and audit logs. Iteration 2 re-validates all 10 original security items after Bug 3 security hardening fixes.

**Verdict:** CONDITIONAL PASS (class: amber)

2026-07-24 · **4 findings** (2 new, 2 inherited) · Security Architect

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 0 |
| Medium | **3** |
| Low | **1** |

> **Security Scope**
> Phase 1 is a local-first storage engine. The attack surface includes the PyO3 Python bridge (data deserialization boundary), the CLI `--db-path` argument, compression decompression paths (compression bomb risk), and key/value injection vectors. Authentication/authorization is out of scope — this layer trusts its caller entirely. Bug 3 addressed 7 of the original 7 findings. This iteration re-verifies all 10 security items and identifies 2 new gaps not caught in the original review.

---

## 02 · Security Items Verification

### 10 Security Items — PASS/FAIL Summary

| # | Item | File | Status |
|---|---|---|---|
| 1 | LZ4 64MB decompress bound | `src/compression/mod.rs:90-106` | ✅ PASS |
| 2 | Zstd 128MB decompress bound | `src/compression/mod.rs:56-74` | ✅ PASS |
| 3 | Memory content >1MB rejection (create) | `src/engine/mod.rs:322-327` | ✅ PASS |
| 4 | Setting key validation (empty, >256 chars) | `src/engine/mod.rs:698-705` | ✅ PASS |
| 5 | EngineError::sanitized() strips IDs | `src/error.rs:54-65` | ✅ PASS |
| 6 | CLI path validation (non-dir → reject) | `src/cli.rs:511-513` | ✅ PASS |
| 7 | CLI path validation (/tmp → warning) | `src/cli.rs:519-523` | ⚠️ WARNING only, not rejection |
| 8 | Skill.file_path doc comment present | `src/types/mod.rs:317-342` | ✅ PASS |
| 9 | PyO3 catch_unwind on all methods | `src/python.rs:70-87`, 26 call sites | ✅ PASS |
| 10 | JSON depth limiting | `src/python.rs:94-101` | ❌ FAIL — `disable_recursion_limit()` |

---

## 03 · Vulnerability Findings

### 🟡 SEC-003 (Medium, inherited) — JSON recursion limit disabled on PyO3 bridge

| Field | Value |
|---|---|
| **File** | `src/python.rs:94-101` |
| **Cargo.toml** | `serde_json` with `unbounded_depth` feature (line 11) |
| **Risk** | Deeply nested JSON payload can cause stack overflow during deserialization |
| **Attack vector** | Any Python caller passing crafted JSON to any bridge method |
| **Original finding** | SEC-003 in Phase 4 report recommended `set_max_depth(32)` |
| **Current implementation** | The custom `from_str` function explicitly **disables** the recursion limit |

The `from_str` helper at `src/python.rs:94-101` is used by every bridge method that accepts JSON. It calls `de.disable_recursion_limit()`:

```rust
fn from_str<T>(s: &str) -> serde_json::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let mut de = serde_json::Deserializer::from_str(s);
    de.disable_recursion_limit();  // <-- explicitly disabled
    T::deserialize(&mut de)
}
```

The doc comment states this was done "to prevent false positives on legitimate deep input while allowing serde_json's streaming parser to handle nesting safely." However, serde_json's recursive descent parser uses the call stack for nested structures. Without a recursion limit, a sufficiently deep payload (e.g., `[[[...]]]` at depth > 512) can overflow the C stack. This contradicts the original SEC-003 recommendation.

**Blast radius:** Every PyO3 method that deserializes JSON from Python callers. An attacker (or untrusted Python code) sending crafted JSON can terminate the Python process with a stack overflow.

**Remediation:** Either (a) restore a sensible recursion limit (e.g., `de.set_max_depth(128)`) and test with production payloads, or (b) add a pre-deserialization size limit (e.g., reject JSON payloads > 10MB before parsing) as a defense-in-depth measure.

---

### 🟡 SEC-008 (Medium, new) — Memory content size limit bypassable via update_memory

| Field | Value |
|---|---|
| **Files** | `src/engine/mod.rs:449-454` (update_memory), `src/storage/rocksdb_backend.rs:856-870` |
| **Risk** | 1MB content limit is only enforced on `create_memory`, but NOT on `update_memory` |
| **Attack vector** | Create a small memory, then update it with arbitrarily large content, bypassing the 1MB limit |

The `create_memory` method at `src/engine/mod.rs:322-327` enforces a 1MB content size limit:

```rust
if new_memory.content.len() > 1024 * 1024 {
    return Err(EngineError::Validation(
        "Memory content exceeds 1MB limit".into(),
    ));
}
```

However, `update_memory` at line 449-454 passes the patch directly to storage without any size validation:

```rust
pub fn update_memory(&self, id: Uuid, patch: &MemoryPatch) -> EngineResult<Memory> {
    let memory = self.storage.write().unwrap().update_memory(id, patch)?;
    let key = memory_cache_key(&id);
    self.cache.invalidate(&key);
    Ok(memory)
}
```

The `MemoryPatch` struct has an `Option<String>` content field. When `content: Some(...)` is provided via the PyO3 bridge or CLI, the RocksDB backend applies it without size checking.

**Blast radius:** Any caller with access to the update_memory API (CLI, PyO3 bridge) can store arbitrarily large memory content, causing resource exhaustion. The 1MB create-time guardrail is trivially bypassed.

**Remediation:** Add the same content size check to `update_memory`:

```rust
pub fn update_memory(&self, id: Uuid, patch: &MemoryPatch) -> EngineResult<Memory> {
    if let Some(ref content) = patch.content {
        if content.len() > 1024 * 1024 {
            return Err(EngineError::Validation(
                "Memory content exceeds 1MB limit".into(),
            ));
        }
    }
    let memory = self.storage.write().unwrap().update_memory(id, patch)?;
    let key = memory_cache_key(&id);
    self.cache.invalidate(&key);
    Ok(memory)
}
```

Also add tests for the update path boundary: 1MB exactly should succeed, >1MB should be rejected.

---

### 🟡 SEC-009 (Medium, new) — CLI /tmp path is a warning, not a rejection

| Field | Value |
|---|---|
| **File** | `src/cli.rs:519-523` |
| **Risk** | User may lose database on reboot if using `/tmp` |
| **Original finding** | SEC-006 in Phase 4 report — classified as Low |

The CLI checks for `/tmp` paths and emits a warning:

```rust
if canonical.starts_with("/tmp") {
    eprintln!(
        "Warning: data in {} may be lost on reboot",
        canonical.display()
    );
}
```

This does NOT block execution — the engine continues to open and operate on the `/tmp` path. For a CLI tool, this is a reasonable UX choice (users may intentionally use `/tmp` for throwaway testing), but it means the "no /tmp" requirement is only advisory.

**Blast radius:** Accidental data loss if a user specifies `--db-path /tmp/contexter` without understanding the reboot semantics.

**Remediation:** If the requirement is strict rejection, change to `eprintln!("Error: ...")` + `std::process::exit(1)`. If warning is acceptable, document this as a known trade-off in the CLI help text.

---

### 🔵 SEC-010 (Low, inherited) — Skill.file_path stored without runtime validation

| Field | Value |
|---|---|
| **File** | `src/types/mod.rs:317-342` |
| **Risk** | Path traversal if file_path is dereferenced in a future phase |
| **Original finding** | SEC-007 in Phase 4 report — addressed with doc comment only |

The `Skill.file_path` field has an excellent doc comment (lines 317-326) that explicitly warns about the path traversal risk. However, the runtime code still accepts any path without:

1. Canonicalization
2. Allow-listing to a safe directory
3. Path traversal component stripping (`../`)

This is acceptable for Phase 1 since `file_path` is only stored/returned, not dereferenced. However, any future phase that loads or executes skills from these paths MUST add validation first.

**Blast radius:** Currently zero (stored only). Future: arbitrary file read/execute if dereferenced without validation.

**Remediation:** None needed in Phase 1. Document this as a **pre-condition** for any Phase 2 work that loads skill files.

---

## 04 · Resolved Findings (from original Phase 4 report)

| Original Finding | Fix Evidence | Status |
|---|---|---|
| SEC-001 (High) — LZ4 no decompress bound | `src/compression/mod.rs:90-106` — 64MB post-decompress check + tests | ✅ RESOLVED |
| SEC-002 (High, partial) — No memory content size limit | `src/engine/mod.rs:322-327` — 1MB limit on create_memory | ⚠️ Partial (see SEC-008 for update path gap) |
| SEC-003 (Medium — serde_json depth) | NOT FIXED — `disable_recursion_limit()` called instead | ❌ UNRESOLVED (re-stated above) |
| SEC-004 (Medium) — Setting key validation | `src/engine/mod.rs:698-705` — empty + >256 char rejection + tests | ✅ RESOLVED |
| SEC-005 (Medium) — Error messages expose IDs | `src/error.rs:54-65` — sanitized() strips all entity IDs + tests | ✅ RESOLVED |
| SEC-006 (Low) — CLI path validation minimal | `src/cli.rs:508-524` — non-dir rejection, canonicalization, /tmp warning | ✅ RESOLVED (as-designed) |
| SEC-007 (Low) — Skill.file_path unsanitized | `src/types/mod.rs:317-342` — doc comment documents risk | ✅ RESOLVED (doc-only) |

---

## 05 · Security-Critical Code Highlights

### Positive — Bug 3 hardening successfully addressed 7 of 7 original findings
- LZ4 + Zstd decompress bounds with tests for oversized and empty edge cases
- Memory content size limit on create (1MB) with boundary test (exactly 1MB passes, 1MB+1 byte rejected)
- Setting key validation (empty/256-char boundary) with 4 tests
- `sanitized()` strips IDs from all `EngineError` variants with 7 tests
- CLI path canonicalization with non-directory rejection
- `Skill.file_path` security doc comment
- `Validation` variant on `EngineError`

### Positive — Every PyO3 method wraps body in catch_unwind
- 26 `#[pymethods]` call sites, all wrapped in `catch_panic()` which uses `catch_unwind(AssertUnwindSafe(f))`
- Prevents Rust panics from unwinding into Python, which would corrupt the Python interpreter state

### Positive — UUID validation at all external boundaries
- Every UUID string from CLI (`parse_uuid()`) and PyO3 bridge (`parse_uuid()`) validated before use
- Invalid inputs rejected with clear error messages

### Positive — RocksDB WAL flushed synchronously on every write
- `flush_wal(true)` called after every write operation across all entity types

### Concern — No size validation on any binary input path
- `create_memory_bytes` (PyO3 bridge) accepts raw `&[u8]` content without size limit
- Storage engine writes binary blobs without size checks
- Should be documented as caller's responsibility or should enforce a limit

---

## 06 · Remediation Recommendations

> **Must Fix**
> - **SEC-008**: Add content size validation to `Engine::update_memory` — check `patch.content.len()` against the same 1MB limit enforced on create_memory. Without this, the create-time guardrail is trivially bypassed.
> - **SEC-003**: Decide on a JSON depth strategy — either restore `set_max_depth(128)` on the `from_str` helper, or add a pre-deserialization payload size limit (e.g., 10MB max) as defense-in-depth. The current `disable_recursion_limit()` approach is the opposite of the original recommendation.

> **Should Fix**
> - **SEC-009**: Either harden `/tmp` check from warning to error+exit, or document the warning-only behavior explicitly in CLI help text (`--help` output) so users are informed before operating.

> **Consider**
> - **SEC-010**: Document `Skill.file_path` as a **blocker** for any Phase 2 work that dereferences it. Add a validation gate in the project roadmap.
> - Add size validation to `create_memory_bytes` and any future binary input paths.
> - Consider adding `cargo audit` and `cargo deny` to CI to catch dependency CVEs automatically.

---

## 07 · Test Coverage Gaps

| Gap | Details |
|---|---|
| `update_memory` content size | No test verifies that updating memory with >1MB content is rejected. Add a test analogous to `test_memory_content_exceeds_limit_rejected` but for the update path. |
| JSON depth / size limits | No test verifies behavior with deeply nested or oversized JSON payloads at the PyO3 bridge. |
| `create_memory_bytes` size | No test verifies size limits on the binary content path. |
| CLI `/tmp` path | No test verifies the warning message is printed (integration test gap). |

---

_Generated by Security Architect · 2026-07-24 · Validation Contract: 2026-07-23-contexter-phase1 (Iteration 2)_
