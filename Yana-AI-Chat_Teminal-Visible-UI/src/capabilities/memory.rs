// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryKind { Working, Session, Project, Decision }

impl MemoryKind {
    pub fn label(self) -> &'static str { match self { Self::Working => "working", Self::Session => "session", Self::Project => "project", Self::Decision => "decision" } }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryFact {
    pub id: u64,
    pub kind: MemoryKind,
    pub subject: String,
    pub value: String,
    pub session: u64,
    pub timestamp: u64,
}

#[derive(Debug, Default)]
pub struct ZeroMemory {
    next_id: u64,
    facts: Vec<MemoryFact>,
    entity_index: BTreeMap<String, BTreeSet<u64>>,
    timeline: BTreeMap<u64, Vec<u64>>,
}

impl ZeroMemory {
    pub fn remember(&mut self, kind: MemoryKind, subject: impl Into<String>, value: impl Into<String>, session: u64, timestamp: u64) -> u64 {
        let subject = subject.into();
        let value = value.into();
        let id = if self.next_id == 0 { 1 } else { self.next_id };
        self.next_id = id + 1;
        self.entity_index.entry(subject.to_ascii_lowercase()).or_default().insert(id);
        self.timeline.entry(timestamp).or_default().push(id);
        self.facts.push(MemoryFact { id, kind, subject, value, session, timestamp });
        id
    }

    pub fn retrieve(&self, query: &str, limit: usize) -> Vec<&MemoryFact> {
        let words = query.to_ascii_lowercase().split_whitespace().map(str::to_owned).collect::<Vec<_>>();
        let mut scored = self.facts.iter().map(|fact| {
            let haystack = format!("{} {}", fact.subject, fact.value).to_ascii_lowercase();
            let score = words.iter().filter(|word| haystack.contains(word.as_str())).count();
            (score, fact)
        }).filter(|(score, _)| *score > 0).collect::<Vec<_>>();
        scored.sort_by(|(left_score, left), (right_score, right)| right_score.cmp(left_score).then_with(|| right.timestamp.cmp(&left.timestamp)).then_with(|| right.id.cmp(&left.id)));
        scored.into_iter().take(limit).map(|(_, fact)| fact).collect()
    }

    pub fn recent(&self, limit: usize) -> Vec<&MemoryFact> { self.facts.iter().rev().take(limit).collect() }
    pub fn len(&self) -> usize { self.facts.len() }
    pub fn counts(&self) -> BTreeMap<MemoryKind, usize> {
        let mut counts = BTreeMap::new();
        for fact in &self.facts { *counts.entry(fact.kind).or_insert(0) += 1; }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retrieval_uses_original_evidence_without_summary() {
        let mut memory = ZeroMemory::default();
        memory.remember(MemoryKind::Decision, "terminal boundary", "Do not duplicate Yana Core", 1, 10);
        memory.remember(MemoryKind::Project, "theme", "Sky Lake", 1, 11);
        let result = memory.retrieve("Yana Core boundary", 3);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].value, "Do not duplicate Yana Core");
    }
}
