// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub id: u64,
    pub path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub reason: String,
    pub timestamp: u64,
    pub author: String,
    pub confidence: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactRecord {
    pub key: String,
    pub value: String,
    pub source: String,
    pub observed_at: u64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EvidenceIndex {
    next_id: u64,
    evidence: Vec<EvidenceRecord>,
    facts: BTreeMap<String, FactRecord>,
}

impl EvidenceIndex {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            ..Self::default()
        }
    }

    pub fn record_evidence(
        &mut self,
        path: impl Into<String>,
        start_line: usize,
        end_line: usize,
        reason: impl Into<String>,
        timestamp: u64,
        author: impl Into<String>,
        confidence: u8,
    ) -> Result<u64, EvidenceError> {
        if start_line == 0 || end_line < start_line {
            return Err(EvidenceError::InvalidRange {
                start_line,
                end_line,
            });
        }
        let id = self.next_id;
        self.next_id += 1;
        self.evidence.push(EvidenceRecord {
            id,
            path: path.into(),
            start_line,
            end_line,
            reason: reason.into(),
            timestamp,
            author: author.into(),
            confidence: confidence.min(100),
        });
        Ok(id)
    }

    pub fn upsert_fact(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
        source: impl Into<String>,
        observed_at: u64,
    ) {
        let key = key.into();
        self.facts.insert(
            key.clone(),
            FactRecord {
                key,
                value: value.into(),
                source: source.into(),
                observed_at,
            },
        );
    }

    pub fn for_path<'a>(&'a self, path: &'a str) -> impl Iterator<Item = &'a EvidenceRecord> {
        self.evidence.iter().filter(move |item| item.path == path)
    }

    pub fn fact(&self, key: &str) -> Option<&FactRecord> {
        self.facts.get(key)
    }

    pub fn strongest_for_path(&self, path: &str) -> Option<&EvidenceRecord> {
        self.for_path(path).max_by_key(|item| item.confidence)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceError {
    InvalidRange { start_line: usize, end_line: usize },
}

impl std::fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRange {
                start_line,
                end_line,
            } => write!(f, "invalid evidence range: {start_line}..={end_line}"),
        }
    }
}

impl std::error::Error for EvidenceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facts_replace_stale_observations_by_key() {
        let mut index = EvidenceIndex::new();
        index.upsert_fact("ui.header.color", "blue", "theme.rs", 1);
        index.upsert_fact("ui.header.color", "pink", "theme.rs", 2);
        assert_eq!(index.fact("ui.header.color").unwrap().value, "pink");
    }
}
