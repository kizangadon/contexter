# Bug 3: Security Hardening

## Problem
Four security gaps identified by the Security Architect validator: LZ4 decompression has no output-size bound (compression bomb), Memory content has no size validation, settings keys have no validation, error messages expose entity IDs, CLI path has no validation, and Skill.file_path has undocumented path traversal risk.

## Root Cause
Insufficient input validation and output sanitization across the codebase.

## Fix Requirements
1. Add 64MB decompressed-size limit to `Lz4Compression::decompress`
2. Add 128MB decompressed-size limit to `ZstdCompression::decompress`
3. Reject memory content > 1MB in `Engine::create_memory`
4. Reject setting keys that are empty or > 256 chars in `Engine::set_setting`
5. Add `EngineError::sanitized()` method that strips entity IDs
6. Add CLI path existence validation and /tmp warning
7. Add doc comment on `Skill.file_path` noting path traversal risk
8. Ensure `Validation` variant exists on `EngineError`
