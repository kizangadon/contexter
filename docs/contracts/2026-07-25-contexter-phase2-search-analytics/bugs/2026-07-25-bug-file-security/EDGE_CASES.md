# Edge Cases — Bug-File-Security

- EC-01: Temp dir already exists with wrong permissions — set permissions on creation only
- EC-02: Snapshot file is a symlink — follow symlink, check target size
- EC-03: Snapshot file is a directory — return IsADirectory error
