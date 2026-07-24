# Bug 15 Design Preview — JSON Depth

## Changes
1. `bridge.rs`: Remove `check_json_depth()` function entirely
2. `bridge.rs`: Simplify `from_str()` to just call `serde_json::from_str(s)` directly
3. Remove `MAX_JSON_DEPTH` constant
