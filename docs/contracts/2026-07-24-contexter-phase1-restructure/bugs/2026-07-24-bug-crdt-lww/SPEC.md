# Bug 23: Implement LWW-Register Struct in crdt/mod.rs

## REQ-CRD-001: Define LwwRegister with logical + wall clock timestamps
`contexter-core/src/crdt/mod.rs` currently is a stub with `pub mod merge;` and a TODO. Add an `LwwRegister<T>` struct with:
- `value: T`
- `logical_clock: u64` (monotonic counter)
- `wall_clock: DateTime<Utc>` (wall clock timestamp)
- A `merge(self, other: LwwRegister<T>) -> LwwRegister<T>` method that delegates to `crdt::merge::lww_merge()`
- A `new(value: T) -> Self` constructor with current wall_clock and logical_clock=0
- A `fn value(&self) -> &T` accessor

Use `chrono::DateTime<Utc>` for wall_clock timestamps (already a dependency).
