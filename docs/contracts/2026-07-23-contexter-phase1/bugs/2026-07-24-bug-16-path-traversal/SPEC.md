# Bug 16: Path traversal prevention for `validate_file_path`

## Description
`Engine::validate_file_path` in `src/engine/mod.rs` only checks empty and length limits.
It does not prevent path components like `..` that could enable directory traversal.
Add a basic guard rejecting paths containing `..` path segments.

## Requirements
- REQ-B16-001: `validate_file_path` SHALL reject paths containing `..` as a path component
- REQ-B16-002: The error message SHALL mention "path traversal"
- REQ-B16-003: The fix SHALL NOT require filesystem I/O (no `canonicalize`)
- REQ-B16-004: The fix SHALL NOT require a design discussion (simple string check)
- REQ-B16-005: Applies to both `create_skill` and `update_skill` entry points
