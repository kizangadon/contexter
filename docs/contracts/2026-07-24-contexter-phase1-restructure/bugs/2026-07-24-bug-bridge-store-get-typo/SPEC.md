# Bug 21: Fix bridge.rs store/get type mismatches

## REQ-SGT-001: Fix fn store — remove .as_bytes()
In `src/bridge.rs:497`, `value.as_bytes()` converts `&str` to `&[u8]`, but `Engine::store()` takes `&str` for value. Change `self.inner.store(cf_name, key, value.as_bytes())` to `self.inner.store(cf_name, key, value)`.

## REQ-SGT-002: Fix fn get — remove from_utf8 decode
In `src/bridge.rs:503`, `String::from_utf8` attempts to decode bytes to String, but `Engine::get()` already returns `Option<String>`. Remove the `from_utf8` wrapping so the result is returned directly.
