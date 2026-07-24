# Bug 14: Remaining Security Issues

## Problems
1. `disable_recursion_limit()` was used to fix PyO3 compilation, but this removes JSON depth protection. Need to implement proper depth limiting.
2. `update_memory()` bypasses the 1MB content size limit that `create_memory()` has.
3. CLI `/tmp` path is warning-only (not a rejection).
4. `Skill.file_path` has no runtime path validation.

## Fix Requirements
1. Implement JSON depth limiting without `set_max_depth` (use a custom Deserializer wrapper or manual depth counting in a visitor)
2. Add the same 1MB content validation to `update_memory()` as `create_memory()` has
3. Evaluate CLI `/tmp` path — if hard rejection is appropriate, make it one
4. Add runtime path validation for `Skill.file_path`
