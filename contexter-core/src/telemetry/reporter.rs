//! Metrics reporter for pushing/exporting telemetry data.
//!
//! (Stub — Phase 2)

/// Reports collected metrics to external sinks (e.g. stdout, OTLP, Prometheus).
#[allow(dead_code)]
pub struct MetricsReporter {
    // TODO(phase2): add export configuration, batch size, interval
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Placeholder test for Phase 2 metrics reporting.
    #[test]
    fn test_placeholder() {
        let _reporter = MetricsReporter {};
    }
}
