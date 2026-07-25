# Bug 5: PyO3 Compilation Fix

## Problem
`cargo test --all-features` produces 23 compilation errors in `src/python.rs`:
1. `serde_json::Deserializer::set_max_depth()` removed in newer serde_json
2. `map_err` type mismatch: EngineError→PyErr closure passed where serde_json::Error expected (22 occurrences)
3. `#[pymodule]`/`add_class` needs `Bound<PyModule>` for PyO3 v0.22+
4. Unused variable `tel` on line ~1014

## Fix Requirements
1. Replace `set_max_depth` with `serde_json::Deserializer::from_str(s).disable_recursion_limit()` or equivalent
2. Fix all 22 `map_err` calls to use proper closure converting `serde_json::Error` to `PyErr`
3. Update `#[pymodule]` to use `Bound<PyModule>` API
4. Remove or prefix unused `tel` variable
