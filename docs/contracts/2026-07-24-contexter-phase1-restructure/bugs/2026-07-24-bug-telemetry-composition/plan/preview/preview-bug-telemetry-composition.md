# Bug 14 Design Preview — Telemetry Composition

## Changes
1. `telemetry/mod.rs`: Add `TelemetryCollector` struct wrapping `EngineStats`
2. `engine/mod.rs`: Add `telemetry: Arc<TelemetryCollector>` to `Engine`, remove standalone `stats`
3. `engine/session.rs`, `memory.rs`, `maintenance.rs`: Replace `self.stats.XXX` with `self.telemetry.stats.XXX`
4. `telemetry/metrics.rs`, `reporter.rs`, `tracing.rs`: Stubs unchanged
