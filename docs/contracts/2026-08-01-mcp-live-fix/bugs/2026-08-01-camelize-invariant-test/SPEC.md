# SPEC — Camelize Collision Invariant Test (iter-1 finding SEC-F04)

## Context
Security validator: `_camelize_payload_keys` collision-free invariant is not asserted by a test
(e.g., keys that map to the same camelCase form, or `foo_bar` vs `fooBar` collisions).

## Requirements
- REQ-CCI-001: Add a test asserting `_camelize_payload_keys` is collision-free on a set of
  adversarial key pairs (snake_case variants that would collide after camelization, e.g.
  `foo_bar` and `fooBar`, `a_b` and `ab`-style traps).
- REQ-CCI-002: Test asserts deterministic ordering if the implementation relies on insertion
  order; document the collision policy in the test (last-wins or error).
- REQ-CCI-003: Full suite passes.
