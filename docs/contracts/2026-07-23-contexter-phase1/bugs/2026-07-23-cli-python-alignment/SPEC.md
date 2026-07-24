# Bug 2: CLI + Python Surface Alignment

## Problem
CLI lacks `status` and `checkpoint` top-level commands. Python bridge lacks `catch_unwind` protection, JSON depth limiting on untrusted input, uses `health()` instead of `status()`, has incorrect return types (bool vs None), and `list_sessions` has wrong signature.

## Root Cause
Spec design and implementation diverged on CLI command layout and Python API signatures.

## Fix Requirements
1. Add `contexter status` top-level command showing path, per-CF sizes, counts, cache ratio
2. Add `contexter checkpoint` top-level command
3. Wrap all `#[pymethod]` bodies with `std::panic::catch_unwind` -> convert panics to `PyErr`
4. Use `serde_json::Deserializer::set_max_depth(64)` for all JSON deserialization in `python.rs`
5. Rename `PyEngine::health()` -> `PyEngine::status()`
6. Change `delete_session`/`delete_memory` return from `PyResult<bool>` to `PyResult<()>` (Python None)
7. Fix `list_sessions` signature: remove offset/limit params (use internal defaults 0, 50)
