# Bug 15: JSON Depth Check — Two-Pass Optimization

## REQ-JSN-001: Eliminate two-pass JSON scanning
Currently, `bridge.rs` `from_str()` (line 117-124) calls `check_json_depth()` first (a full character-by-character scan), then calls `serde_json::from_str()` (a second full parse). This does two complete passes over the input.

**Fix**: Either:
1. Integrate depth tracking into the `serde_json::Deserializer` recursion limit by setting `serde_json::Deserializer::new(s).disable_recursion_limit()` and relying on `serde_json`'s built-in recursion protection, OR
2. Remove the `check_json_depth` separate pass and instead apply depth tracking inside `serde_json`'s streaming parser using `serde_json::StreamDeserializer`, OR
3. **Simplest**: Move the depth check into `serde_json`'s own deserialization by using `serde_json::Deserializer::from_reader` with a custom recursion limiter.

**Simplest actual fix**: Since `serde_json` already has recursion protection via its `RecursionLimit`, just remove the manual `check_json_depth` pre-scan entirely. `serde_json::from_str()` has a default recursion limit of 128, which is more generous than the current 64 and sufficient for all use cases. The pre-scan provides no additional safety.
