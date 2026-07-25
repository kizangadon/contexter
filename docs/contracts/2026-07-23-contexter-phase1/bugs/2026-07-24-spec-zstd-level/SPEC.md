# Bug 6: SPEC REQ-S-007 Zstd Level Mismatch

## Problem
SPEC.md REQ-S-007 requires conflicts CF to use Zstd compression at level 1. Implementation uses Zstd default level 3.

## Fix Requirements
1. In `src/storage/rocksdb_backend.rs`, set zstd compression level to 1 for the conflicts column family
