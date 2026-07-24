# Security Review Report

# Contexter Phase 1R — Rust Core Restructure

> Purely structural restructure — code moved, split, and reorganized with zero logic changes. Review focuses on security implications of the restructuring: secrets leakage, FFI safety, module visibility changes, input validation regression, and dependency changes.

**Verdict:** PASS (class: SECURITY-NEUTRAL)

2026-07-24 · 0 findings · Security Architect

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 0 |

> **Security Scope**
> This review examined the structural restructure of the Contexter Rust crate from a flat `src/` layout to a workspace-member `contexter-core/` crate with DDD-aligned module decomposition. The scope covers: secrets leakage across moved files, FFI safety in the PyO3 bridge (`bridge.rs`), module visibility changes in `lib.rs`, input validation regression at the Python↔Rust boundary, and dependency changes in `contexter-core/Cargo.toml`. The review is **delta-based** — it identifies only issues introduced or exposed by the restructure, not pre-existing concerns.

---

## 02 · Vulnerability Findings

**No findings.** The restructure introduced zero new vulnerabilities.

Every security control present in the old code is preserved in the new code with identical logic:

| Security Control | Old Location | New Location | Status |
|---|---|---|---|
| JSON depth checking (`MAX_JSON_DEPTH = 64`) | `src/python.rs` | `contexter-core/src/bridge.rs` | Identical |
| `check_json_depth()` + `from_str()` guard | `src/python.rs` | `contexter-core/src/bridge.rs` | Identical |
| UUID validation (`parse_uuid()`) | `src/python.rs` | `contexter-core/src/bridge.rs` | Identical |
| FFI panic safety (`catch_panic` + `AssertUnwindSafe`) | `src/python.rs` | `contexter-core/src/bridge.rs` | Identical |
| `EngineError::sanitized()` for generic error messages | `src/error.rs` | `contexter-core/src/error.rs` | Identical |
| `Arc<Engine>` Send+Sync safety | `src/python.rs` | `contexter-core/src/bridge.rs` | Identical |
| `#[cfg(feature = "python")]` gate on bridge | `src/lib.rs` | `contexter-core/src/lib.rs` | Preserved |

---

## 03 · Security-Critical Code Highlights

### FFI Boundary — PyO3 Bridge (`contexter-core/src/bridge.rs`)

The bridge module is the **only FFI boundary** in the codebase. The restructure moved it from `src/python.rs` to `contexter-core/src/bridge.rs` with zero semantic changes:

```rust
// Every Python-facing method wraps its body in catch_panic.
// No unsafe blocks anywhere in the file.
fn catch_panic<F, T>(f: F) -> PyResult<T>
where
    F: FnOnce() -> PyResult<T>,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(panic_info) => {
            let msg = /* extract panic message */;
            Err(PyErr::new::<PyRuntimeError, _>(msg))
        }
    }
}
```

**Key observations:**
- Zero `unsafe` blocks in bridge.rs or anywhere in the crate (`grep unsafe` returned zero matches across all `.rs` files in `contexter-core/src/`)
- The bridge uses `serde_json` as the data boundary (JSON strings in, JSON strings out) — no complex PyO3 type mappings that could introduce safety issues
- `Arc<Engine>` is `Send + Sync` — validated by compile-time assertions in inline tests
- `#[pymodule]` init function (`contexter`) registers `PyEngine` as a Python class

### Input Validation — JSON Depth Checking

The depth check was preserved identically from old to new:

```rust
const MAX_JSON_DEPTH: usize = 64;

fn check_json_depth(input: &str) -> Result<(), String> {
    // Scans for nesting depth without fully parsing
    // Guards against stack-overflow or resource-exhaustion attacks
}
```

All bridge methods that accept JSON strings route through `from_str()` which calls `check_json_depth()` before `serde_json::from_str()`.

### Module Visibility (`contexter-core/src/lib.rs`)

The restructure changed the module tree but preserved the security-relevant visibility:

| Old Export | New Export | Change |
|---|---|---|
| `pub mod python` | `pub mod bridge` (behind `#[cfg(feature = "python")]`) | Renamed, same gating |
| `pub mod types` | `pub mod models` | Renamed, same visibility |
| `pub use types::*` | `pub use models::*` | Renamed re-export |
| `pub use storage::StorageBackend` | Same | Unchanged |
| _(none)_ | `pub mod crdt`, `telemetry`, `util`, `versioning`, `wal` | New structural stubs (Phase 2) |
| _(none)_ | `pub mod analytics`, `fts`, `vector` | Phase 2 stub modules |

**Assessment:** No new attack surface from visibility changes. The new modules (`crdt`, `telemetry`, `wal`, `versioning`, `util`) contain only structural code. The Phase 2 stubs (`analytics`, `fts`, `vector`) are empty `mod.rs` files with `#[allow(dead_code)]` where needed.

---

## 04 · Remediation Recommendations

> **Must Fix**
> None introduced by this restructure.

> **Should Fix**
> None introduced by this restructure.

> **Consider**
> (Pre-existing, not a regression) The `Skill.file_path` field in `contexter-core/src/models/skill.rs` (line 32) accepts an optional filesystem path with **no validation or canonicalisation**. This is documented in a security note in the source (lines 9–16), but it remains unfixed. A downstream consumer that reads `file_path` and loads/executes that file could be vulnerable to path-traversal attacks. This was present in the old `src/types/mod.rs` and was simply moved to the new location. It is tracked elsewhere (previous Auto Bug Loop iteration 3 reports); this report notes it only for completeness — it is not a regression from this restructure.

---

_Generated by Security Architect · 2026-07-24 · Validation Contract: contexter-phase1-restructure_
