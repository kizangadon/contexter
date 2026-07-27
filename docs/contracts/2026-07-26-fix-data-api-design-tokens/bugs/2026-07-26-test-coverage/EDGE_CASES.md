# Edge Cases — Test Coverage

## E-TEST-01: Expanduser with explicit non-tilde path
If path is already absolute (e.g., `/tmp/test`), `os.path.expanduser` is a no-op — test must confirm this does not break.

## E-TEST-02: Role with explicit None
`role: Optional[str] = Field(default="system")` — if someone passes `role=None`, it should be `None`, not `"system"`. The test must verify this.
