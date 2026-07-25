//! Column family definitions and key encoding for the Contexter storage engine.

// ---------------------------------------------------------------------------
// Column family names
// ---------------------------------------------------------------------------

/// Name of the primary memory-storage column family.
pub const CF_MEMORY_ITEMS: &str = "memory_items";
/// Name of the session-storage column family.
pub const CF_SESSIONS: &str = "sessions";
/// Name of the agent-definitions column family.
pub const CF_AGENTS: &str = "agents";
/// Name of the skill-definitions column family.
pub const CF_SKILLS: &str = "skills";
/// Name of the efficiency-map column family.
pub const CF_EFFICIENCY_MAP: &str = "efficiency_map";
/// Name of the telemetry column family.
pub const CF_TELEMETRY: &str = "telemetry";
/// Name of the conflict-records column family.
pub const CF_CONFLICTS: &str = "conflicts";
/// Name of the index-metadata column family.
pub const CF_INDEX_STATE: &str = "index_state";
/// Name of the secondary-memory-index column family.
pub const CF_MEMORY_INDEX: &str = "memory_index";
/// Name of the settings column family (CFA-001).
pub const CF_SETTINGS: &str = "settings";
/// Name of the audit-log column family (CFA-002).
pub const CF_AUDIT: &str = "audit";
/// Name of the session-secondary-index column family (CFA-003).
pub const CF_SESSION_INDEX: &str = "session_index";

// ---------------------------------------------------------------------------
// Key prefixes (pub so Engine can use them for chunked iteration)
// ---------------------------------------------------------------------------

pub const KEY_PREFIX_SESSION: &str = "ses:";
pub const KEY_PREFIX_MEMORY: &str = "mem:";
pub const KEY_PREFIX_AGENT: &str = "agt:";
pub const KEY_PREFIX_SKILL: &str = "skl:";
pub const KEY_PREFIX_SETTING: &str = "cfg:";
pub const KEY_PREFIX_AUDIT: &str = "aud:";

/// Holds the names of all 12 column families.
///
/// Each field stores the static CF name string. Use
/// [`super::rocksdb::RocksDbBackend::cf`] to resolve a name to a [`ColumnFamily`] reference.
///
/// `#[allow(dead_code)]` — fields provide forwards-compatible access to CF
/// names even if some are not yet referenced from every code path.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColumnFamilyMap {
    /// CF name for [`CF_MEMORY_ITEMS`].
    pub memory_items: &'static str,
    /// CF name for [`CF_SESSIONS`].
    pub sessions: &'static str,
    /// CF name for [`CF_AGENTS`].
    pub agents: &'static str,
    /// CF name for [`CF_SKILLS`].
    pub skills: &'static str,
    /// CF name for [`CF_EFFICIENCY_MAP`].
    pub efficiency_map: &'static str,
    /// CF name for [`CF_TELEMETRY`].
    pub telemetry: &'static str,
    /// CF name for [`CF_CONFLICTS`].
    pub conflicts: &'static str,
    /// CF name for [`CF_INDEX_STATE`].
    pub index_state: &'static str,
    /// CF name for [`CF_MEMORY_INDEX`].
    pub memory_index: &'static str,
    /// CF name for [`CF_SETTINGS`].
    pub settings: &'static str,
    /// CF name for [`CF_AUDIT`].
    pub audit: &'static str,
    /// CF name for [`CF_SESSION_INDEX`].
    pub session_index: &'static str,
}

impl Default for ColumnFamilyMap {
    fn default() -> Self {
        Self::new()
    }
}

impl ColumnFamilyMap {
    /// Create a [`ColumnFamilyMap`] with the canonical CF names.
    pub const fn new() -> Self {
        Self {
            memory_items: CF_MEMORY_ITEMS,
            sessions: CF_SESSIONS,
            agents: CF_AGENTS,
            skills: CF_SKILLS,
            efficiency_map: CF_EFFICIENCY_MAP,
            telemetry: CF_TELEMETRY,
            conflicts: CF_CONFLICTS,
            index_state: CF_INDEX_STATE,
            memory_index: CF_MEMORY_INDEX,
            settings: CF_SETTINGS,
            audit: CF_AUDIT,
            session_index: CF_SESSION_INDEX,
        }
    }

    /// Iterate over all CF names.
    pub fn iter(&self) -> impl Iterator<Item = &'static str> + '_ {
        [
            self.memory_items,
            self.sessions,
            self.agents,
            self.skills,
            self.efficiency_map,
            self.telemetry,
            self.conflicts,
            self.index_state,
            self.memory_index,
            self.settings,
            self.audit,
            self.session_index,
        ]
        .into_iter()
    }

    /// Return all registered CF names as a `Vec<String>`.
    pub fn cf_names(&self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_family_map_contains_all_cfs() {
        let map = ColumnFamilyMap::new();
        let names: Vec<&str> = map.iter().collect();
        assert!(names.contains(&CF_MEMORY_ITEMS));
        assert!(names.contains(&CF_SESSIONS));
        assert!(names.contains(&CF_AGENTS));
        assert!(names.contains(&CF_SKILLS));
        assert!(names.contains(&CF_EFFICIENCY_MAP));
        assert!(names.contains(&CF_TELEMETRY));
        assert!(names.contains(&CF_CONFLICTS));
        assert!(names.contains(&CF_INDEX_STATE));
        assert!(names.contains(&CF_MEMORY_INDEX));
        assert!(names.contains(&CF_SETTINGS));
        assert!(names.contains(&CF_AUDIT));
        assert!(names.contains(&CF_SESSION_INDEX));
        assert_eq!(names.len(), 12);
    }

    #[test]
    fn cf_names_returns_all() {
        let map = ColumnFamilyMap::new();
        let names = map.cf_names();
        assert_eq!(names.len(), 12);
        assert!(names.contains(&CF_SETTINGS.to_string()));
        assert!(names.contains(&CF_AUDIT.to_string()));
        assert!(names.contains(&CF_SESSION_INDEX.to_string()));
    }
}
