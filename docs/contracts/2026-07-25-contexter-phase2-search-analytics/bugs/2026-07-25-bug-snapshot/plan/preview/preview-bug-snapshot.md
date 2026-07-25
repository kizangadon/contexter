# Design Preview — Bug-Snapshot

## Fix Plan
1. Implement `save(path)` on `HnswVectorIndex` — serialize `layers`, `entry_point`, and metadata via bincode
2. Implement `periodic_snapshot(interval_secs, token)` — tokio::spawn loop with tokio::time::interval
3. Add `shutdown()` method to Engine that calls save on vector_index and joins snapshot handle
