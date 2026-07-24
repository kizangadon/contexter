# Bug 17: Remove serde_json unbounded_depth Feature

## REQ-UBD-001: Remove unbounded_depth feature flag
Remove `features = ["unbounded_depth"]` from the `serde_json` dependency in `contexter-core/Cargo.toml`. This feature disables serde_json's built-in recursion limit (default 128), which with the removal of `check_json_depth()` (Bug 15) creates a stack-overflow DoS vulnerability via deeply nested JSON input. serde_json's default recursion limit of 128 is sufficient for all application use cases.
