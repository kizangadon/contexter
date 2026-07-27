# Bug SPEC: Add regression tests for expanduser + role default

## Context
Two testing gaps were identified in Auto Bug Loop iteration 1:

1. **S-03**: `test_bridge.py` has no test that verifies tilde expansion works. All init tests use explicit paths like `/tmp/test-contexter` — none test `~/.contexter/` expansion. This is a regression risk: if `os.path.expanduser()` were removed or broken, the tilde path would silently create an empty DB in the wrong location.

2. **N-04**: No test verifies the `role` field default behavior. The `role: Optional[str] = Field(default="system")` default is a design decision (D-A3) but has no test coverage.

## Design Decisions (Resolved)
- **S-01** (MemoryCreate/SessionCreate not updated): This was deliberate per D-A2. The Create models are input-only and receive data from Python code, not Rust JSON. No change needed. ✓
- **S-02** (role default "system"): This was deliberate per D-A3. The default is documented and safe for imported rekal data. No change needed. ✓
- **N-01** (hex casing): `#181716` is lowercase in the V2-DEEP spec. This is spec-compliant. No change needed. ✓
- **F-01** (embedding vectors exposed): Noted as documentation gap. Embedding exposure in API is an accepted architectural trade-off for this feature scope. ✓
- **F-02** (UTC timezone consistency): rekal data is already in UTC. No conversion needed. ✓

## Fix
Add two tests to `contexter-server/tests/test_bridge.py`:
1. `test_os_expanduser_called`: Verify that `os.path.expanduser` is called during bridge initialization with a tilde path
2. `test_role_default_is_system`: Verify that creating a Memory without specifying `role` results in `role == "system"`

## Acceptance Criteria (AC-TEST-01 through AC-TEST-04)
- Test 1: tilde expansion — bridge init with `~/.contexter/` calls expanduser
- Test 2: tilde expansion — path is resolved before passing to Rust engine
- Test 3: role default — Memory without `role` field has `role == "system"`
- Test 4: role default — explicit `role=None` also produces `role == None` (Optional allows explicit None)
