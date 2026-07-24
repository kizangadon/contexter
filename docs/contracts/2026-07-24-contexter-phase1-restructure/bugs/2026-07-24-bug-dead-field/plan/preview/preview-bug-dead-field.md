# Bug 12 Design Preview — Dead Field

## Changes
1. `src/models/memory.rs`: Add `#[serde(skip)]` + `#[allow(dead_code)]` to `MemorySearchQuery.project` field
