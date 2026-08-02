# ACCEPTANCE — Camelization Live-Coverage Tests

## AC-CM-001
GIVEN the fixed bridge
WHEN a live-engine coverage run executes
THEN all 34 engine methods are exercised and their responses parse against Python models (report of 34/34, or documented exceptions)

## AC-CM-002
GIVEN the coverage test
THEN every method without a live test has a documented reason and a shape-locked mock test

## AC-CM-003
GIVEN the fix
THEN full suite ≥647 passed / 1 known pre-existing
