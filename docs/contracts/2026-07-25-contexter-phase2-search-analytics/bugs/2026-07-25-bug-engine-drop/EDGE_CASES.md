# Edge Cases

1. **Shutdown while snapshot in progress** — thread may be in the middle of `save_snapshot`; signal cancellation and wait
2. **Double drop** — Rust prevents this but the Drop impl should still be safe
3. **No snapshot thread** — `snapshot_handle` is `None`; `take()` handles this
4. **Panic in shutdown** — `Drop` should not panic; catch errors and log
5. **Racing with periodic snapshot** — cancellation is atomic bool; ensure memory ordering is correct
