# Edge Cases

1. **Zero-length string** — length prefix of 0 should produce an empty string, not an error
2. **Maximum valid length** — exactly 1024 bytes should succeed
3. **Existing valid snapshots** — must still load correctly after changes
4. **Memory-mapped files** — `metadata()` on opened handles works with mmap
5. **Read-only filesystem** — opening for metadata should still work
