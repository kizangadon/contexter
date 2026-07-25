# Bug 17: JSON depth hardening (test + safety comments)

## Description
Two trivial items in `src/python.rs`:
1. **F-02**: Add a boundary test for `check_json_depth` at exactly 65 levels of nesting
2. **F-03**: Add `// SAFETY:` comments on 6 direct `serde_json::from_str` calls explaining why depth checking is unnecessary (internal data, bounded nesting)

## Requirements
- REQ-B17-001: Add `test_json_depth_exceeds_limit` test asserting depth-65 is rejected
- REQ-B17-002: Add `// SAFETY:` comments on lines 708, 798, 918, 1053, 1088, 1160 in `src/python.rs`
- REQ-B17-003: All existing tests still pass
