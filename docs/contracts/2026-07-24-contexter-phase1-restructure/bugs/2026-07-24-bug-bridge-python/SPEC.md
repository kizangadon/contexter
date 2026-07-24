# Bug 18: Fix Bridge Python Feature Compile Errors

## REQ-BPY-001: Fix hit_ratio field
In `src/bridge.rs` line 522, replace `tel.hit_ratio` with a computed value: `if tel.total_ops > 0 { tel.hits as f64 / tel.total_ops as f64 } else { 0.0 }`. The `CacheTelemetry` struct does not have a `hit_ratio` field.
