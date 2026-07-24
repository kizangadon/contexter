//! Last-writer-wins (LWW) merge logic for CRDT reconciliation.
//!
//! (Stub — Phase 2)

/// Performs a last-writer-wins merge between two values using wall-clock
/// timestamps as the tiebreaker.
///
/// When timestamps are equal the left-hand value is preferred.
#[allow(dead_code)]
pub fn lww_merge<T>(left: T, _right: T, left_time: i64, right_time: i64) -> T {
    if left_time >= right_time {
        left
    } else {
        _right
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the LWW merge prefers the newer timestamp.
    #[test]
    fn test_lww_merge_prefers_newer() {
        let result = lww_merge("old", "new", 100, 200);
        assert_eq!(result, "new");
    }

    /// Verify that the LWW merge prefers left when timestamps are equal.
    #[test]
    fn test_lww_merge_equal_timestamps() {
        let result = lww_merge("left", "right", 42, 42);
        assert_eq!(result, "left");
    }
}
