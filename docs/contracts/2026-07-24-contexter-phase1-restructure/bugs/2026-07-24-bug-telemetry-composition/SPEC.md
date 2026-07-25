# Bug 14: Telemetry Composition in Engine

## REQ-TEL-001: Engine composites telemetry module
The `Engine` struct (engine/mod.rs:156-160) has a `stats: EngineStats` field but does not reference the `telemetry` module at all. The telemetry module is a Phase 2 stub that contains `metrics`, `reporter`, and `tracing` sub-modules.

**Fix**: Add a `telemetry` field to the `Engine` struct that holds the telemetry subsystems. Since the telemetry modules are stubs, this is a structural change:
1. Add `pub mod telemetry` to `engine/mod.rs` (or import from `crate::telemetry`)
2. Add a `telemetry: Arc<TelemetryCollector>` field to `Engine`
3. Create a simple `TelemetryCollector` struct in the telemetry module that holds the engine stats
4. Wire it through `Engine::open` and `Engine::with_config`
5. Remove the standalone `stats: EngineStats` in favor of the telemetry' integrated stats

**Simplified approach**: Since telemetry is a stub, the minimal fix is:
1. Create `pub struct TelemetryCollector { pub stats: EngineStats }` in `telemetry/mod.rs`
2. Add `telemetry: Arc<TelemetryCollector>` to `Engine`
3. Initialize in `open`/`with_config`
4. Route all existing `self.stats.XXX` calls through `self.telemetry.stats.XXX`
5. Remove the standalone `stats` field from `Engine`
