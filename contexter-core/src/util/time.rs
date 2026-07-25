//! Timestamp helpers for wall-clock and monotonic time.

use chrono::{DateTime, Utc};

/// Returns the current UTC timestamp.
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Returns the current Unix timestamp in milliseconds.
pub fn now_millis() -> i64 {
    Utc::now().timestamp_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that `now()` returns a recent timestamp.
    #[test]
    fn test_now_is_recent() {
        let ts = now();
        let elapsed = Utc::now().signed_duration_since(ts);
        assert!(elapsed.num_seconds() < 5);
    }

    /// Verify that `now_millis()` returns a positive value.
    #[test]
    fn test_now_millis_is_positive() {
        let ms = now_millis();
        assert!(ms > 1_700_000_000_000); // well past early 2023
    }
}
