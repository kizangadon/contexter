# Security Review Report

# Contexter Phase 1R Restructure — Auto Bug Loop Iteration 2

> Re-validation of bug fix iteration 2: RocksDB safety (Bug 8), module structure (Bug 9), missing tests (Bug 10), bridge API (Bug 11), dead field (Bug 12), CF architecture (Bug 13), telemetry (Bug 14), JSON depth (Bug 15), engine tests (Bug 16). Reviews all applied fixes for security regressions and new vulnerabilities introduced by the changes.

**Verdict:** FAIL (class: HAS-FINDINGS)

2026-07-24 · 5 findings · Security Architect

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 1 |
| High | 2 |
| Medium | 1 |
| Low | 1 |

> **Security Scope**
> This review examined 9 bug fixes applied across the Rust and Python bridge codebase:
> - Bug 8: `cf()` returns `EngineResult` instead of panicking; `maybe_flush_wal()` added to `store_raw` and `write_batch`
> - Bug 9: `error.rs` → `error/mod.rs`, `cli.rs` → `cli/mod.rs` (structural, security-neutral)
> - Bug 10: 5 new integration test files (test-only, security-neutral)
> - Bug 11: Bridge `store()` takes `&str`, `get()` returns `Option<String>`
> - Bug 12: `#[serde(skip)]` on unused `MemorySearchQuery.project` field
> - Bug 13: 3 new column families (`CF_SETTINGS`, `CF_AUDIT`, `CF_SESSION_INDEX`), session secondary index
> - Bug 14: `TelemetryCollector` wraps `EngineStats`, `Engine.telemetry` field replaces standalone `stats`
> - Bug 15: `check_json_depth()` removed; `serde_json::from_str()` called directly
> - Bug 16: Inline engine tests extracted to `tests/engine/` files

---

## 02 · Vulnerability Findings

### SEC-CRIT-001 — JSON Depth Protection Disabled (Stack-Overflow DoS)

**Severity:** Critical  
**Affected files:**
- `contexter-core/Cargo.toml` (line 11) — `features = ["unbounded_depth"]`
- `contexter-core/src/bridge.rs` (lines 67–72) — `from_str()` calls `serde_json::from_str(s)` directly
- `contexter-core/src/bridge.rs` (lines 102, 175, 186, 215, 227, 252, 276, 289, 326, 354, 391, 431, 439) — all 13 bridge endpoints that deserialize user JSON input

**Issue:**
Bug 15 removed the manual `check_json_depth()` pre-scan (MAX_JSON_DEPTH = 64) under the rationale that `serde_json` "already has recursion protection via its RecursionLimit, default 128." However, the `Cargo.toml` still enables the `unbounded_depth` feature, which **explicitly disables** serde_json's recursion limit. The result: **no depth protection at all** on any user-supplied JSON payload.

An attacker can send a JSON payload with nesting depth > 1500 (e.g., `[[[[...]]]]`) and cause a stack overflow in the Rust deserializer, crashing the process (DoS).

**Attack scenario:**
```
# A call to PyEngine::create_session() with deeply nested JSON:
echo '[[[[[[[[...x1600...]]]]]]]]' | python3 -c "
import json, contexter
e = contexter.Engine.open('/tmp/test')
e.create_session('{...}')  # JSON deserialization in bridge.rs:102
"
# → Stack overflow, Rust panic unwinding across FFI → process crash
```

**Impact:** Denial of service. An unauthenticated caller (or authenticated user) can crash the engine process with a small payload.

**Recommended fix — choose one:**
1. **Remove `unbounded_depth` from `Cargo.toml`** — rely on serde_json's default recursion limit of 128, which is more permissive than the original 64 and sufficient for all documented use cases.
   ```toml
   # Cargo.toml line 11
   serde_json = { version = "1", features = ["v7", "serde"] }
   ```
2. **Re-add a depth check** — restore `check_json_depth()` with a reasonable limit (128) and keep `unbounded_depth` for safety on serde_json's streaming deserializer.

**Fix verification:** After removing `unbounded_depth`, verify `serde_json::from_str()` rejects deeply nested input:
```rust
#[test]
fn test_deeply_nested_json_rejected() {
    let deep = format!("[{}]", "[".repeat(200));
    let result: serde_json::Result<serde_json::Value> = serde_json::from_str(&deep);
    assert!(result.is_err(), "deeply nested JSON should be rejected");
}
```

---

### SEC-HIGH-001 — Bridge store() Type Mismatch (Python Feature Compile Error)

**Severity:** High  
**Affected file:** `contexter-core/src/bridge.rs` (line 497)
```rust
catch_panic(|| self.inner.store(cf_name, key, value.as_bytes()).map_err(map_err))
//                                                    ^^^^^^^^^^^^^^^^
//                                                    &[u8] but Engine::store() expects &str
```

**Issue:**
Bug 11 changed `Engine::store()` signature from `value: &[u8]` to `value: &str`. However, `PyEngine::store()` still passes `value.as_bytes()` (a `&[u8]`). This is a type mismatch that causes a **compile error** when building with the `python` feature enabled:

```
error[E0308]: mismatched types
   --> contexter-core/src/bridge.rs:497:55
    |
497 |         catch_panic(|| self.inner.store(cf_name, key, value.as_bytes()).map_err(map_err))
    |                                   -----               ^^^^^^^^^^^^^^^^ expected `&str`, found `&[u8]`
```

**Impact:** The Python feature cannot be compiled, blocking deployment for Python consumers.

**Recommended fix:**
```rust
// bridge.rs line 497 — pass &str directly, no .as_bytes()
catch_panic(|| self.inner.store(cf_name, key, value).map_err(map_err))
```

---

### SEC-HIGH-002 — Bridge get() String::from_utf8 Type Mismatch (Python Feature Compile Error)

**Severity:** High  
**Affected file:** `contexter-core/src/bridge.rs` (lines 500–506)
```rust
fn get(&self, cf_name: &str, key: &str) -> PyResult<Option<String>> {
    catch_panic(|| {
        self.inner.get(cf_name, key).map_err(map_err)
            .map(|opt| opt.map(String::from_utf8).transpose())
//                                ^^^^^^^^^^^^^^^^^
//                                String::from_utf8 expects Vec<u8>,
//                                but Engine::get() now returns Option<String>
            .and_then(|r| r.map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("invalid UTF-8: {e}"))))
    })
}
```

**Issue:**
Bug 11 changed `Engine::get()` to return `EngineResult<Option<String>>`. The bridge's `get()` method applies `String::from_utf8` on the result, but `String::from_utf8` expects `Vec<u8>`, not `String`. This is a type mismatch that causes a **compile error**:

```
error[E0631]: type mismatch in function arguments
   --> contexter-core/src/bridge.rs:503:36
    |
503 |                 .map(|opt| opt.map(String::from_utf8).transpose())
    |                                --- ^^^^^^^^^^^^^^^^^
    |                                |   |
    |                                |   expected fn(String) -> _
    |                                found fn(Vec<u8>) -> _
```

**Impact:** The Python feature cannot be compiled, blocking deployment for Python consumers.

**Recommended fix:**
```rust
// bridge.rs lines 500-506 — remove the String::from_utf8 conversion, result is already String
fn get(&self, cf_name: &str, key: &str) -> PyResult<Option<String>> {
    catch_panic(|| {
        self.inner.get(cf_name, key).map_err(map_err)
    })
}
```

---

### SEC-MED-001 — Bridge status() References Non-Existent `hit_ratio` Field

**Severity:** Medium  
**Affected file:** `contexter-core/src/bridge.rs` (line 522)
```rust
"hitRatio": tel.hit_ratio,
//               ^^^^^^^^^
//               CacheTelemetry has no field `hit_ratio`
```

**Issue:**
The `status()` method in bridge.rs references `tel.hit_ratio`, but `CacheTelemetry` (defined in `contexter-core/src/cache/metrics.rs:8`) does not have a `hit_ratio` field. The struct has: `gets`, `hits`, `misses`, `stores`, `invalidations`, `total_ops`, and `entries_by_type`. This causes a **compile error**:

```
error[E0609]: no field `hit_ratio` on type `CacheTelemetry`
   --> contexter-core/src/bridge.rs:522:37
    |
522 |                     "hitRatio": tel.hit_ratio,
    |                                     ^^^^^^^^^ unknown field
```

**Impact:** The Python feature cannot be compiled. Additionally, the `status` endpoint is the primary health-check API for Python consumers.

**Recommended fix:**
```rust
// bridge.rs lines 514-529 — compute hit ratio from available fields or remove
let hit_ratio = if tel.total_ops > 0 {
    tel.hits as f64 / tel.total_ops as f64
} else {
    0.0
};
// OR use the hit_ratio computed in cli.rs:1114 pattern
```

---

### SEC-LOW-001 — RwLock unwrap() in Production Code Paths

**Severity:** Low  
**Affected files:** All engine sub-module files:
- `contexter-core/src/engine/session.rs` (lines 21, 40, 56, 62, 78, 108, 119, 129)
- `contexter-core/src/engine/memory.rs` (lines 24, 42, 68, 79)
- `contexter-core/src/engine/agent.rs` (lines 18, 36, 52, 58, 107, 117)
- `contexter-core/src/engine/skill.rs` (lines 45, 63, 79, 85, 126, 136)
- `contexter-core/src/engine/settings.rs` (lines 26, 46, 62, 72, 78)
- `contexter-core/src/engine/search.rs` (lines 23, 30)
- `contexter-core/src/engine/maintenance.rs` (lines 15, 21, 26, 52, 59)

**Issue:**
Every engine method acquires the `SharedBackend` lock via `self.storage.write().unwrap()` or `self.storage.read().unwrap()`. If a thread panics while holding the write lock, the `RwLock` becomes poisoned, and **all subsequent calls** to `.unwrap()` on that `RwLock` will panic.

This is a pre-existing pattern (not introduced by this iteration), but it is documented here for completeness. The blast radius is limited: if a write-lock-holding thread panics, the entire engine becomes inoperable until the process restarts.

**Impact:** A panic in any storage operation poisons the RwLock, rendering the engine unusable for all threads. This amplifies the blast radius of any other panic-causing bug.

**Recommended fix (deferred):**
Replace `.unwrap()` with `.expect("RwLock poisoned: ...")` with a descriptive message, or handle poison by recreating the lock:

```rust
// Alternative: handle poisoned lock
self.storage.write().unwrap_or_else(|poisoned| poisoned.into_inner())
```

This is a defensive hardening pattern. Consider implementing as part of a separate hardening pass.

---

## 03 · Security-Critical Code Highlights

### A. Bug 8 — `cf()` Returns Result (✅ Fixed, No Regressions)

```rust
// rocksdb.rs:198 — now returns EngineResult instead of panicking
fn cf(&self, name: &str) -> EngineResult<&ColumnFamily> {
    self.db
        .cf_handle(name)
        .ok_or_else(|| EngineError::Storage(format!("column family '{name}' not found")))
}
```

- No panic path for missing CFs ✅
- All 17 call sites use `self.cf(cf_name)?` to propagate errors ✅

### B. Bug 8 — `maybe_flush_wal()` Coverage Verified

Both `store_raw()` (rocksdb.rs:1399) and `write_batch()` (rocksdb.rs:1416) now call `self.maybe_flush_wal()?` after writing, matching all other mutating methods. WAL durability is consistent across all write paths. ✅

### C. Bug 13 — Column Family Creation Verified

- 12 CFs defined in `column_families.rs` (8 original + 3 new + `CF_MEMORY_INDEX`)
- All 12 registered in `ColumnFamilyMap::new()` ✅
- All 12 configured in `open_with_config()` with compression settings ✅
- `cf_names` returns all 12 ✅
- `create_missing_column_families(true)` in `open_with_config()` ✅
- `ColumnFamilyMap` struct has `#[allow(dead_code)]` for forward-compat ✅

### D. Bug 14 — Telemetry Composition Verified

- `TelemetryCollector` created in `telemetry/mod.rs` wrapping `EngineStats` ✅
- `Engine.telemetry: Arc<TelemetryCollector>` field replaces `Engine.stats` ✅
- All `self.stats.XXX` calls routed through `self.telemetry.stats.XXX` ✅
- `TelemetryCollector` is `Send + Sync` (tested in `test_telemetry_collector_is_send_sync`) ✅
- No mutex contention — all counters use `AtomicU64` ✅

### E. Bug 11 — Bridge `store()`/`get()` String Handling

**⚠️ Compile errors (see SEC-HIGH-001 and SEC-HIGH-002)**

- `Engine::get()` (maintenance.rs:56-67) properly handles UTF-8 errors using `map_err` + `transpose()` ✅
- `Engine::store()` (maintenance.rs:50-53) takes `&str` and converts to bytes safely ✅
- Bridge `store()` (bridge.rs:496-498) has type mismatch — passes `&[u8]` to `&str` ❌
- Bridge `get()` (bridge.rs:500-506) has type mismatch — applies `String::from_utf8` to `String` ❌

### F. Unsafe Code Audit

| Check | Result |
|---|---|
| `unsafe` keyword in production code | ❌ Not found |
| `unsafe` keyword in any source file | ❌ Not found |
| `extern "C"` blocks | ❌ Not found (PyO3 handles FFI) |

**Conclusion:** The codebase is 100% safe Rust. No unsafe blocks in any production or test code.

### G. New Column Families and Index Safety

The 3 new CFs (`CF_SETTINGS`, `CF_AUDIT`, `CF_SESSION_INDEX`) are:
- Created via `create_missing_column_families(true)` during `open_with_config()`
- Named via `&'static str` constants (no allocation at lookup time)
- Accessed through `self.cf(name)?` which returns `EngineResult`
- The session secondary index uses an `idx:session:{project}:{agent_id}:{status}:{uuid}` key format with proper prefix scanning in `list_sessions` and `count_sessions`

The secondary index is read-only during normal operations (sessions are written through `create_session`/`update_session`/`delete_session` which maintain the index). No unbounded iteration or unvalidated key material.

---

## 04 · Remediation Recommendations

> **Must Fix**
> 1. **SEC-CRIT-001**: Remove `unbounded_depth` from `serde_json` features in `Cargo.toml` (line 11), OR re-add `check_json_depth()` guard. Without this, any user-supplied JSON payload can cause a stack-overflow crash.
> 2. **SEC-HIGH-001**: Fix `bridge.rs:497` — pass `value` (not `value.as_bytes()`) to `Engine::store()`. Currently a compile error under `python` feature.
> 3. **SEC-HIGH-002**: Fix `bridge.rs:500-506` — remove spurious `String::from_utf8` conversion. `Engine::get()` already returns `Option<String>`. Currently a compile error under `python` feature.
> 4. **SEC-MED-001**: Fix `bridge.rs:522` — replace `tel.hit_ratio` with computed value or remove the field. `CacheTelemetry` has no `hit_ratio` field. Currently a compile error under `python` feature.

> **Should Fix**
> 1. Add `#[cfg(not(feature = "python"))]` or conditional compilation guards to bridge-only types so the `python` feature compiles independently. Currently, enabling `python` introduces 3 compilation errors.

> **Consider**
> 1. **RwLock poison handling** (SEC-LOW-001): Replace `.unwrap()` with `.expect()` on `RwLock` acquisitions to provide diagnostic messages if a thread panics while holding the lock.
> 2. **Integration test for `cargo check --features python`**: Add a CI step that runs `cargo check --features python` to catch bridge compilation errors before they reach review.
> 3. **Depth limit validation test**: Add a test that verifies `serde_json` rejects deeply nested input (after removing `unbounded_depth`).

---

## 05 · Summary of Bug Fix Security Verification

| Bug | Fix Applied | Security Impact | Status |
|---|---|---|---|
| Bug 8 — RocksDB safety | `cf()` returns Result; `maybe_flush_wal()` on all write paths | ✅ Positive — eliminates panic path, ensures WAL durability | ✅ Verified |
| Bug 9 — Module structure | `error.rs` → `error/mod.rs`, `cli.rs` → `cli/mod.rs` | ✅ Neutral — structural only | ✅ Verified |
| Bug 10 — Missing tests | 5 new integration test files | ✅ Neutral — test-only | ✅ Verified |
| Bug 11 — Bridge API | `store(&str)` / `get() -> Option<String>` | ❌ **2 compile errors** — bridge not updated to match new signatures | ❌ FAIL |
| Bug 12 — Dead field | `#[serde(skip)]` on `MemorySearchQuery.project` | ✅ Neutral — unused field removed from serialization | ✅ Verified |
| Bug 13 — CF architecture | 3 new CFs, session secondary index | ✅ Positive — proper separation of concerns, no uninitialized CF access | ✅ Verified |
| Bug 14 — Telemetry | `TelemetryCollector` wraps `EngineStats` | ✅ Neutral — Arc-wrapped, atomic counters, Send+Sync | ✅ Verified |
| Bug 15 — JSON depth | `check_json_depth()` removed | ❌ **Critical** — `unbounded_depth` still enabled, no depth guard | ❌ FAIL |
| Bug 16 — Engine tests | Inline tests extracted to `tests/engine/` | ✅ Neutral — test-only | ✅ Verified |

**Pass rate:** 7/9 bugs correctly fixed; 2 bugs introduced regressions (Bug 11: 2 compile errors, Bug 15: 1 critical DoS vulnerability).

---

_Generated by Security Architect · 2026-07-24 · Validation Contract: contexter-phase1-restructure_
