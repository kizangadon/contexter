# Bug 11: Bridge API Type Compliance

## REQ-BRG-001: Bridge store() accepts &str value
Change `PyEngine::store()` signature from `value: Vec<u8>` to `value: &str`. The generic raw storage bridge is designed for Python callers that serialize to JSON strings. Binary value support is not required at this level.

Change `Engine::store()` (the method called by bridge) to accept `&str` as well.

## REQ-BRG-002: Bridge get() returns Option<String>
Change `PyEngine::get()` return type from `Option<Vec<u8>>` to `Option<String>`.

Change `Engine::get()` to return `Option<String>` for consistency.

**NOTE**: `StorageBackend` trait's `store()` and `get()` methods retain their `&[u8]`/`Option<Vec<u8>>` signatures for internal use. Only the bridge-facing methods are changed.
