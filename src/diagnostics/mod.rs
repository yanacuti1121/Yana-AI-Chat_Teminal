// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct DiagnosticBundle {
    max_entries: usize,
    entries: BTreeMap<String, String>,
}

impl DiagnosticBundle {
    pub fn with_limit(max_entries: usize) -> Self {
        Self {
            max_entries: max_entries.max(1),
            entries: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        if !self.entries.contains_key(&key) && self.entries.len() >= self.max_entries {
            return;
        }
        self.entries.insert(key, redact(value.into()));
    }

    pub fn entries(&self) -> impl Iterator<Item = DiagnosticEntry> + '_ {
        self.entries.iter().map(|(key, value)| DiagnosticEntry {
            key: key.clone(),
            value: value.clone(),
        })
    }

    pub fn render_text(&self) -> String {
        self.entries
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn redact(value: String) -> String {
    let lower = value.to_ascii_lowercase();
    if ["api_key", "apikey", "authorization", "bearer ", "token=", "secret="]
        .iter()
        .any(|marker| lower.contains(marker))
    {
        "[REDACTED]".to_owned()
    } else {
        value
    }
}

impl Default for DiagnosticBundle {
    fn default() -> Self {
        Self::with_limit(128)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_secret_like_values() {
        let mut bundle = DiagnosticBundle::default();
        bundle.insert("provider", "Authorization: Bearer abc123");
        assert!(bundle.render_text().contains("[REDACTED]"));
        assert!(!bundle.render_text().contains("abc123"));
    }

    #[test]
    fn respects_entry_limit() {
        let mut bundle = DiagnosticBundle::with_limit(1);
        bundle.insert("a", "1");
        bundle.insert("b", "2");
        assert_eq!(bundle.entries().count(), 1);
    }
}
