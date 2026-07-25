//! Predefined analytical SQL query templates for the DuckDB engine.
//!
//! Each constant is a complete SQL query with `?` placeholders for parameters.
//! These are used by [`DuckDbEngine`](crate::analytics::duckdb::DuckDbEngine)
//! for predefined analysis operations.

/// SQL query for session count by time range.
///
/// Parameters: `?1` = start timestamp, `?2` = end timestamp.
pub const SESSION_COUNT_BY_RANGE: &str = "
    SELECT DATE(created_at) as day, COUNT(*) as count
    FROM sessions
    WHERE created_at >= ? AND created_at <= ?
    GROUP BY day
    ORDER BY day
";

/// SQL query for memory count by type.
pub const MEMORY_COUNT_BY_TYPE: &str = "
    SELECT memory_type, COUNT(*) as count
    FROM memories
    GROUP BY memory_type
    ORDER BY count DESC
";

/// SQL query for telemetry aggregation.
pub const TELEMETRY_AGGREGATION: &str = "
    SELECT event_type, scope,
           COUNT(*) as event_count,
           AVG(value) as avg_value,
           MIN(value) as min_value,
           MAX(value) as max_value
    FROM telemetry
    GROUP BY event_type, scope
    ORDER BY event_count DESC
";

/// SQL query for session efficiency score.
///
/// Efficiency = useful_memories / total_memories per session.
pub const EFFICIENCY_SCORES: &str = "
    SELECT s.id as session_id,
           s.project,
           COUNT(m.id) as total_memories,
           SUM(CASE WHEN m.memory_type = 'preference' THEN 1 ELSE 0 END) as useful_memories,
           CASE WHEN COUNT(m.id) > 0
                THEN CAST(SUM(CASE WHEN m.memory_type = 'preference' THEN 1 ELSE 0 END) AS DOUBLE)
                     / CAST(COUNT(m.id) AS DOUBLE)
                ELSE 0.0
           END as efficiency_score
    FROM sessions s
    LEFT JOIN memories m ON m.session_id = s.id
    GROUP BY s.id, s.project
    ORDER BY efficiency_score DESC
";

/// SQL query for metric correlation (Pearson) between duration and memory count.
pub const METRIC_CORRELATION: &str = "
    WITH stats AS (
        SELECT s.id,
               s.duration_ms,
               COUNT(m.id) as memory_count,
               AVG(CAST(s.duration_ms AS DOUBLE)) OVER() as avg_dur,
               AVG(CAST(COUNT(m.id) AS DOUBLE)) OVER() as avg_mem
        FROM sessions s
        LEFT JOIN memories m ON m.session_id = s.id
        GROUP BY s.id, s.duration_ms
    ),
    covar AS (
        SELECT SUM((duration_ms - avg_dur) * (memory_count - avg_mem)) as covariance,
               SQRT(SUM((duration_ms - avg_dur) * (duration_ms - avg_dur))) as std_dur,
               SQRT(SUM((memory_count - avg_mem) * (memory_count - avg_mem))) as std_mem,
               COUNT(*) as n
        FROM stats
    )
    SELECT CASE WHEN std_dur > 0 AND std_mem > 0
                THEN covariance / (std_dur * std_mem)
                ELSE 0.0
           END as pearson_r,
           n as sample_count
    FROM covar
";

#[cfg(test)]
mod tests {
    use super::*;

    /// Every query constant must be non-empty and contain a SELECT statement.
    #[test]
    fn test_query_constants_non_empty() {
        let queries = [
            ("SESSION_COUNT_BY_RANGE", SESSION_COUNT_BY_RANGE),
            ("MEMORY_COUNT_BY_TYPE", MEMORY_COUNT_BY_TYPE),
            ("TELEMETRY_AGGREGATION", TELEMETRY_AGGREGATION),
            ("EFFICIENCY_SCORES", EFFICIENCY_SCORES),
            ("METRIC_CORRELATION", METRIC_CORRELATION),
        ];

        for (name, sql) in &queries {
            assert!(!sql.is_empty(), "query '{}' must not be empty", name);
            assert!(
                sql.to_uppercase().contains("SELECT"),
                "query '{}' must contain SELECT",
                name
            );
        }
    }

    /// Verify SQL syntax by checking that SELECT and FROM are both present.
    #[test]
    fn test_queries_have_select_and_from() {
        let queries = [
            SESSION_COUNT_BY_RANGE,
            MEMORY_COUNT_BY_TYPE,
            TELEMETRY_AGGREGATION,
            EFFICIENCY_SCORES,
            METRIC_CORRELATION,
        ];

        for (i, sql) in queries.iter().enumerate() {
            let upper = sql.to_uppercase();
            assert!(upper.contains("SELECT"), "query {} missing SELECT", i);
            assert!(upper.contains("FROM"), "query {} missing FROM", i);
            // Aggregation queries should have GROUP BY
            if *sql != SESSION_COUNT_BY_RANGE {
                assert!(upper.contains("GROUP BY"), "query {} missing GROUP BY", i);
            }
        }
    }
}
