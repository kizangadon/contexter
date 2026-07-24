//! UUID generation helpers.
//!
//! Uses the `uuid` crate (v7 time-ordered UUIDs) for all identifier
//! generation.

use uuid::Uuid;

/// Generate a new time-ordered UUID (v7).
pub fn new_id() -> Uuid {
    Uuid::now_v7()
}

/// Generate a hyphenated UUID string.
pub fn new_id_string() -> String {
    new_id().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that generated IDs are non-nil.
    #[test]
    fn test_new_id_is_non_nil() {
        let id = new_id();
        assert!(!id.is_nil());
    }

    /// Verify that generated ID strings are non-empty.
    #[test]
    fn test_new_id_string_is_non_empty() {
        let s = new_id_string();
        assert!(!s.is_empty());
        assert_eq!(s.len(), 36); // standard UUID format
    }
}
