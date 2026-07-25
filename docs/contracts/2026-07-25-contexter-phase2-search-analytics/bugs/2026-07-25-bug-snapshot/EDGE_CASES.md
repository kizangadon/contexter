# Edge Cases — Bug-Snapshot

- EC-01: Snapshot path is read-only — return PermissionDenied error
- EC-02: Load from corrupted file — return CorruptedSnapshot error
- EC-03: Periodic snapshot during concurrent index/memory write — thread-safe via existing Arc<RwLock>
- EC-04: CancellationToken is cancelled — snapshot task shuts down gracefully
- EC-05: Engine::shutdown() called when no vector index — no-op
