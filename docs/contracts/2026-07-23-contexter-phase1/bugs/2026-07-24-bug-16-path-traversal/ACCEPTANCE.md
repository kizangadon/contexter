# Acceptance Criteria — Bug 16

## AC-B16-001: `..` path component rejected
- Given: A `file_path` containing `../` or `/../` or `..` as a component
- When: `create_skill` is called with this path
- Then: An error is returned mentioning "path traversal"

## AC-B16-002: Valid paths still accepted
- Given: A `file_path` like `/home/skills/test.py`
- When: `create_skill` is called
- Then: The skill is created successfully

## AC-B16-003: Update also validates
- Given: An existing skill with a valid path is updated with a path containing `..`
- When: `update_skill` is called
- Then: An error is returned mentioning "path traversal"

## AC-B16-004: None path still accepted
- Given: A `file_path` of `None`
- When: `create_skill` is called
- Then: The skill is created successfully
