use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Internal data model for configuration values.
///
/// Supports two access patterns:
/// - **Flat** — top-level `KEY = value` (`.env`, root of JSON/TOML/YAML)
/// - **Sectioned** — `[section] key = value` (INI, nested TOML/JSON/YAML)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigMap {
    flat: IndexMap<String, String>,
    sections: IndexMap<String, IndexMap<String, String>>,
}

impl ConfigMap {
    /// Create an empty map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // ── insert ────────────────────────────────────────────────────────────────

    /// Insert a flat (top-level) key/value pair.
    pub fn insert_flat(&mut self, key: String, value: String) {
        self.flat.insert(key, value);
    }

    /// Insert a sectioned key/value pair.
    pub fn insert_sectioned(&mut self, section: String, key: String, value: String) {
        self.sections.entry(section).or_default().insert(key, value);
    }

    // ── get ───────────────────────────────────────────────────────────────────

    /// Look up a flat key.
    pub fn get_flat(&self, key: &str) -> Option<&str> {
        self.flat.get(key).map(String::as_str)
    }

    /// Look up `key` inside `section`.
    pub fn get_in_section(&self, section: &str, key: &str) -> Option<&str> {
        self.sections
            .get(section)
            .and_then(|s| s.get(key))
            .map(String::as_str)
    }

    /// Return all key/value pairs for `section`.
    #[must_use]
    pub fn get_section(&self, section: &str) -> Option<&IndexMap<String, String>> {
        self.sections.get(section)
    }

    /// True if `section` exists.
    #[must_use]
    pub fn has_section(&self, section: &str) -> bool {
        self.sections.contains_key(section)
    }

    // ── iteration ─────────────────────────────────────────────────────────────

    /// All flat key/value pairs in insertion order.
    pub fn flat_iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.flat.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// All sections, each yielding its key/value pairs.
    pub fn sections(&self) -> impl Iterator<Item = (&str, &IndexMap<String, String>)> {
        self.sections.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Total number of top-level keys (flat + section names).
    #[must_use]
    pub fn len(&self) -> usize {
        self.flat.len() + self.sections.len()
    }

    /// True if there are no flat keys and no sections.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.flat.is_empty() && self.sections.is_empty()
    }

    /// True if the flat map contains `key` or any section contains `key`.
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        if self.flat.contains_key(key) {
            return true;
        }
        self.sections.values().any(|s| s.contains_key(key))
    }
}
