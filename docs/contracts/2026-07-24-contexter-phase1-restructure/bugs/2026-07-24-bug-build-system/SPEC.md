# Bug: Build system issues

## Problem
1. **Orphaned Cargo.lock**: `contexter-core/Cargo.lock` exists alongside workspace root `Cargo.lock` — should be deleted (only workspace root lockfile should exist)
2. **zstd-sys build dependency**: `zstd-sys` via bindgen needs clang headers that aren't installed. Currently requires manual env vars (`BINDGEN_EXTRA_CLANG_ARGS`, `LIBCLANG_PATH`) to build. The `.cargo/config.toml` at workspace root should make this permanent.

## Requirements
- REQ-001: Delete `contexter-core/Cargo.lock` (stale, 35KB, differs from root)
- REQ-002: Verify `.cargo/config.toml` at workspace root contains the env var fix for zstd-sys:
  ```toml
  [env]
  BINDGEN_EXTRA_CLANG_ARGS = "-I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/local/include -I/usr/include/x86_64-linux-gnu -I/usr/include"
  LIBCLANG_PATH = "/usr/lib/x86_64-linux-gnu/"
  ```
- REQ-003: `cargo build` from repo root must succeed WITHOUT manual env vars
