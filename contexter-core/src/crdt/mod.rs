//! Conflict-free replicated data types (CRDTs) for synchronisation.

pub mod merge;

use chrono::{DateTime, Utc};
use std::cmp::Ordering;

/// A Last-Writer-Wins Register with logical + wall clock timestamps.
#[derive(Debug, Clone)]
pub struct LwwRegister<T> {
    pub value: T,
    pub logical_clock: u64,
    pub wall_clock: DateTime<Utc>,
}

impl<T> LwwRegister<T> {
    /// Create a new register with the given value.
    pub fn new(value: T) -> Self {
        LwwRegister {
            value,
            logical_clock: 0,
            wall_clock: Utc::now(),
        }
    }

    /// Get a reference to the stored value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Merge two registers: higher logical_clock wins, then wall_clock as tiebreaker.
    pub fn merge(self, other: LwwRegister<T>) -> LwwRegister<T> {
        match self.logical_clock.cmp(&other.logical_clock) {
            Ordering::Greater => self,
            Ordering::Less => other,
            Ordering::Equal => {
                if self.wall_clock >= other.wall_clock {
                    self
                } else {
                    other
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_register() {
        let reg: LwwRegister<i32> = LwwRegister::new(42);
        assert_eq!(*reg.value(), 42);
        assert_eq!(reg.logical_clock, 0);
    }

    #[test]
    fn test_merge_higher_logical_clock_wins() {
        let a = LwwRegister {
            value: "old",
            logical_clock: 1,
            wall_clock: Utc::now(),
        };
        let b = LwwRegister {
            value: "new",
            logical_clock: 5,
            wall_clock: Utc::now(),
        };
        let merged = a.merge(b);
        assert_eq!(*merged.value(), "new");
    }

    #[test]
    fn test_merge_tiebreaker_wall_clock() {
        let early = Utc::now();
        let later = early + chrono::Duration::seconds(1);
        let a = LwwRegister {
            value: "earlier",
            logical_clock: 1,
            wall_clock: early,
        };
        let b = LwwRegister {
            value: "later",
            logical_clock: 1,
            wall_clock: later,
        };
        let merged = a.merge(b);
        assert_eq!(*merged.value(), "later");
    }
}