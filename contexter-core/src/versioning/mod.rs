//! Version tracking and optimistic concurrency for entity mutations.
//!
//! (Stub — Phase 2)

pub mod diff;
pub mod gc;
pub mod store;

// TODO(phase2): implement version vector / clock

#[cfg(test)]
mod tests {
    /// Placeholder test for Phase 2 versioning module.
    #[test]
    fn test_placeholder() {
        // Stub — no functionality yet
    }
}
