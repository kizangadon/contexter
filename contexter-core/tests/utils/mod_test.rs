//! Integration tests for the utility module.
//!
//! Tests UUID generation (`util::id`) and timestamp helpers (`util::time`)
//! at the integration level.

use contexter_core::util;

#[path = "../common/mod.rs"]
mod common;

// ---------------------------------------------------------------------------
// ID generation tests
// ---------------------------------------------------------------------------

/// Verify that `new_id()` returns a non-nil UUID.
#[test]
fn test_new_id_is_non_nil() {
    let id = util::id::new_id();
    assert!(!id.is_nil(), "generated UUID should not be nil");
}

/// Verify that `new_id()` returns a UUID v7 (version nibble = 7).
#[test]
fn test_new_id_is_v7() {
    let id = util::id::new_id();
    let bytes = id.as_bytes();
    let version = bytes[6] >> 4;
    assert_eq!(version, 7, "expected UUID version 7, got {version}");
}

/// Verify that consecutive calls to `new_id()` produce different values.
#[test]
fn test_new_id_unique() {
    let id1 = util::id::new_id();
    let id2 = util::id::new_id();
    assert_ne!(id1, id2, "consecutive UUIDs should differ");
}

/// Verify that `new_id_string()` returns a valid UUID string.
#[test]
fn test_new_id_string_format() {
    let s = util::id::new_id_string();
    assert_eq!(s.len(), 36, "UUID string should be 36 characters");
    // Verify the format: 8-4-4-4-12 hex digits.
    let parts: Vec<&str> = s.split('-').collect();
    assert_eq!(parts.len(), 5, "UUID should have 5 hyphen-separated parts");
    assert_eq!(parts[0].len(), 8);
    assert_eq!(parts[1].len(), 4);
    assert_eq!(parts[2].len(), 4);
    assert_eq!(parts[3].len(), 4);
    assert_eq!(parts[4].len(), 12);
    // All hex characters.
    for part in &parts {
        assert!(part.chars().all(|c| c.is_ascii_hexdigit()), "each part should be hex: {part}");
    }
}

/// Verify that `new_id_string()` parses back to a valid UUID.
#[test]
fn test_new_id_string_parseable() {
    let s = util::id::new_id_string();
    let parsed: uuid::Uuid = s.parse().expect("string should parse as UUID");
    assert!(!parsed.is_nil());
}

/// Verify that multiple IDs generated in sequence are parseable.
#[test]
fn test_multiple_ids_parseable() {
    for _ in 0..100 {
        let s = util::id::new_id_string();
        let parsed: uuid::Uuid = s.parse().expect("string should parse as UUID");
        assert!(!parsed.is_nil());
    }
}

// ---------------------------------------------------------------------------
// Time helpers
// ---------------------------------------------------------------------------

/// Verify that `now()` returns a recent timestamp.
#[test]
fn test_now_is_recent() {
    let ts = util::time::now();
    let elapsed = chrono::Utc::now().signed_duration_since(ts);
    assert!(
        elapsed.num_seconds() < 5,
        "timestamp should be within last 5 seconds"
    );
}

/// Verify that `now_millis()` returns a positive value beyond early 2023.
#[test]
fn test_now_millis_is_positive() {
    let ms = util::time::now_millis();
    assert!(ms > 1_700_000_000_000, "millis should be well past early 2023");
}

/// Verify that `now_millis()` returns consistent values for immediate calls.
#[test]
fn test_now_millis_monotonic() {
    let a = util::time::now_millis();
    let b = util::time::now_millis();
    // b should be >= a (allowing for same-millisecond calls).
    assert!(
        b >= a,
        "second call to now_millis should not be less than first: {b} < {a}"
    );
}

/// Verify that `now()` returns timestamps close to the system clock.
#[test]
fn test_now_close_to_system_clock() {
    let ts = util::time::now();
    let system_now = chrono::Utc::now();
    let diff = (system_now - ts).num_seconds().abs();
    assert!(
        diff < 5,
        "timestamp should be within 5 seconds of system clock, got {diff}s"
    );
}

/// Verify serialization of DateTime<Utc> round-trips correctly.
#[test]
fn test_now_serializable() {
    let ts = util::time::now();
    let json = serde_json::to_string(&ts).expect("serialize timestamp");
    let deserialized: chrono::DateTime<chrono::Utc> =
        serde_json::from_str(&json).expect("deserialize timestamp");
    // The deserialized time should be within a few seconds of the original.
    let diff = (ts - deserialized).num_seconds().abs();
    assert!(diff < 5, "timestamps should match within 5 seconds");
}
