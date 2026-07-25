//! Analytics aggregation for Contexter engine.
//!
//! Provides methods that delegate to the DuckDB-backed L5 analytics engine
//! for efficiency scoring, metric correlation, and data aggregation.

use serde::Serialize;

use crate::analytics::queries;
use crate::analytics::Value;
use crate::engine::Engine;
use crate::error::{EngineError, EngineResult};

// ---------------------------------------------------------------------------
// Analytics result types
// ---------------------------------------------------------------------------

/// Results of an analytics run.
#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsReport {
    pub efficiency_scores: Vec<SessionEfficiency>,
    pub correlation: MetricCorrelation,
    pub session_count_by_type: Vec<(String, u64)>,
    pub memory_count_by_type: Vec<(String, u64)>,
}

/// Efficiency score for a single session.
#[derive(Debug, Clone, Serialize)]
pub struct SessionEfficiency {
    pub session_id: String,
    pub project: String,
    pub total_memories: u64,
    pub useful_memories: u64,
    pub efficiency_score: f64,
}

/// Pearson correlation result between session duration and memory count.
#[derive(Debug, Clone, Serialize)]
pub struct MetricCorrelation {
    pub pearson_r: f64,
    pub sample_count: u64,
}

// ---------------------------------------------------------------------------
// Engine analytics methods
// ---------------------------------------------------------------------------

impl Engine {
    /// Run full analytics and return a comprehensive report.
    ///
    /// Syncs all tables, then runs efficiency scores, metric correlation,
    /// session counts by type (status), and memory counts by type.
    pub fn run_analytics(&self) -> EngineResult<AnalyticsReport> {
        let ae = self
            .analytics_engine
            .as_ref()
            .ok_or_else(|| EngineError::Unimplemented("Analytics not enabled".into()))?;

        // Sync all tables first.
        ae.sync_all()
            .map_err(|e| EngineError::Internal(format!("Analytics sync: {e}")))?;

        // Collect each report component.
        let efficiency_scores = self.get_efficiency_scores_inner(Some(ae.as_ref()))?;
        let correlation = self.get_metric_correlation_inner(Some(ae.as_ref()))?;
        let memory_count_by_type = self.get_memory_count_by_type_inner(Some(ae.as_ref()))?;

        // Session count by type (status) — use a custom query for status-based grouping.
        let session_count_by_type = {
            let raw = ae
                .query(
                    "SELECT status, COUNT(*) as count FROM sessions GROUP BY status ORDER BY count DESC",
                    &[],
                )
                .map_err(|e| EngineError::Internal(format!("Session count query: {e}")))?;
            raw.iter()
                .map(|row| {
                    let status = match &row[0] {
                        Value::Text(s) => s.clone(),
                        _ => "unknown".into(),
                    };
                    let count = match &row[1] {
                        Value::Int(n) => *n as u64,
                        _ => 0,
                    };
                    (status, count)
                })
                .collect()
        };

        Ok(AnalyticsReport {
            efficiency_scores,
            correlation,
            session_count_by_type,
            memory_count_by_type,
        })
    }

    /// Get per-session efficiency scores.
    ///
    /// Efficiency = useful_memories (preferences) / total_memories per session.
    pub fn get_efficiency_scores(&self) -> EngineResult<Vec<SessionEfficiency>> {
        self.get_efficiency_scores_inner(None)
    }

    /// Get Pearson correlation between session duration and memory count.
    pub fn get_metric_correlation(&self) -> EngineResult<MetricCorrelation> {
        self.get_metric_correlation_inner(None)
    }

    /// Get session count grouped by time range.
    pub fn get_session_count_by_range(
        &self,
        start: &str,
        end: &str,
    ) -> EngineResult<Vec<(String, u64)>> {
        let ae = self
            .analytics_engine
            .as_ref()
            .ok_or_else(|| EngineError::Unimplemented("Analytics not enabled".into()))?;

        let raw = ae
            .query(
                queries::SESSION_COUNT_BY_RANGE,
                &[Value::Text(start.into()), Value::Text(end.into())],
            )
            .map_err(|e| EngineError::Internal(format!("Session count range: {e}")))?;

        let results = raw
            .iter()
            .map(|row| {
                let day = match &row[0] {
                    Value::Text(s) => s.clone(),
                    _ => String::new(),
                };
                let count = match &row[1] {
                    Value::Int(n) => *n as u64,
                    _ => 0,
                };
                (day, count)
            })
            .collect();

        Ok(results)
    }

    /// Get memory count grouped by memory type.
    pub fn get_memory_count_by_type(&self) -> EngineResult<Vec<(String, u64)>> {
        self.get_memory_count_by_type_inner(None)
    }

    /// Get telemetry aggregation results.
    pub fn get_telemetry_aggregation(&self) -> EngineResult<Vec<Vec<Value>>> {
        let ae = self
            .analytics_engine
            .as_ref()
            .ok_or_else(|| EngineError::Unimplemented("Analytics not enabled".into()))?;

        let raw = ae
            .query(queries::TELEMETRY_AGGREGATION, &[])
            .map_err(|e| EngineError::Internal(format!("Telemetry agg: {e}")))?;

        Ok(raw)
    }

    // -----------------------------------------------------------------------
    // Internal helpers (accept optional pre-resolved engine ref)
    // -----------------------------------------------------------------------

    fn get_efficiency_scores_inner(
        &self,
        ae_opt: Option<&dyn crate::analytics::AnalyticsEngine>,
    ) -> EngineResult<Vec<SessionEfficiency>> {
        let ae = ae_opt
            .or_else(|| self.analytics_engine.as_deref())
            .ok_or_else(|| EngineError::Unimplemented("Analytics not enabled".into()))?;

        let raw = ae
            .query(queries::EFFICIENCY_SCORES, &[])
            .map_err(|e| EngineError::Internal(format!("Efficiency query: {e}")))?;

        let results = raw
            .iter()
            .map(|row| {
                let session_id = match &row[0] {
                    Value::Text(s) => s.clone(),
                    _ => String::new(),
                };
                let project = match &row[1] {
                    Value::Text(s) => s.clone(),
                    _ => String::new(),
                };
                let total_memories = match &row[2] {
                    Value::Int(n) => *n as u64,
                    _ => 0,
                };
                let useful_memories = match &row[3] {
                    Value::Int(n) => *n as u64,
                    _ => 0,
                };
                let efficiency_score = match &row[4] {
                    Value::Float(f) => *f,
                    Value::Int(n) => *n as f64,
                    _ => 0.0,
                };
                SessionEfficiency {
                    session_id,
                    project,
                    total_memories,
                    useful_memories,
                    efficiency_score,
                }
            })
            .collect();

        Ok(results)
    }

    fn get_metric_correlation_inner(
        &self,
        ae_opt: Option<&dyn crate::analytics::AnalyticsEngine>,
    ) -> EngineResult<MetricCorrelation> {
        let ae = ae_opt
            .or_else(|| self.analytics_engine.as_deref())
            .ok_or_else(|| EngineError::Unimplemented("Analytics not enabled".into()))?;

        let raw = ae
            .query(queries::METRIC_CORRELATION, &[])
            .map_err(|e| EngineError::Internal(format!("Correlation query: {e}")))?;

        // The query returns one row with pearson_r and sample_count.
        if let Some(row) = raw.first() {
            let pearson_r = match &row[0] {
                Value::Float(f) => *f,
                Value::Int(n) => *n as f64,
                _ => 0.0,
            };
            let sample_count = match &row[1] {
                Value::Int(n) => *n as u64,
                _ => 0,
            };
            Ok(MetricCorrelation {
                pearson_r,
                sample_count,
            })
        } else {
            Ok(MetricCorrelation {
                pearson_r: 0.0,
                sample_count: 0,
            })
        }
    }

    fn get_memory_count_by_type_inner(
        &self,
        ae_opt: Option<&dyn crate::analytics::AnalyticsEngine>,
    ) -> EngineResult<Vec<(String, u64)>> {
        let ae = ae_opt
            .or_else(|| self.analytics_engine.as_deref())
            .ok_or_else(|| EngineError::Unimplemented("Analytics not enabled".into()))?;

        let raw = ae
            .query(queries::MEMORY_COUNT_BY_TYPE, &[])
            .map_err(|e| EngineError::Internal(format!("Memory count query: {e}")))?;

        let results = raw
            .iter()
            .map(|row| {
                let memory_type = match &row[0] {
                    Value::Text(s) => s.clone(),
                    _ => String::new(),
                };
                let count = match &row[1] {
                    Value::Int(n) => *n as u64,
                    _ => 0,
                };
                (memory_type, count)
            })
            .collect();

        Ok(results)
    }
}
