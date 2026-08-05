// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalKind {
    Observe,
    Think,
    Plan,
    Decide,
    Act,
    Verify,
    Reflect,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEntry {
    pub sequence: u64,
    pub timestamp: u64,
    pub kind: JournalKind,
    pub summary: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayStep {
    pub offset: u64,
    pub sequence: u64,
    pub kind: JournalKind,
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Default)]
pub struct Journal {
    next_sequence: u64,
    entries: Vec<JournalEntry>,
}

impl Journal {
    pub fn new() -> Self {
        Self {
            next_sequence: 1,
            entries: Vec::new(),
        }
    }

    pub fn record(
        &mut self,
        timestamp: u64,
        kind: JournalKind,
        summary: impl Into<String>,
        detail: impl Into<String>,
    ) -> u64 {
        let sequence = self.next_sequence;
        self.next_sequence += 1;
        self.entries.push(JournalEntry {
            sequence,
            timestamp,
            kind,
            summary: summary.into(),
            detail: detail.into(),
        });
        sequence
    }

    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }

    pub fn since(&self, sequence: u64) -> impl Iterator<Item = &JournalEntry> {
        self.entries
            .iter()
            .filter(move |entry| entry.sequence >= sequence)
    }

    pub fn latest(&self) -> Option<&JournalEntry> {
        self.entries.last()
    }

    pub fn reflection(&self) -> impl Iterator<Item = &JournalEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.kind == JournalKind::Reflect)
    }

    pub fn replay(&self) -> Vec<ReplayStep> {
        let Some(first) = self.entries.first() else {
            return Vec::new();
        };

        self.entries
            .iter()
            .map(|entry| ReplayStep {
                offset: entry.timestamp.saturating_sub(first.timestamp),
                sequence: entry.sequence,
                kind: entry.kind,
                label: entry.summary.clone(),
                detail: entry.detail.clone(),
            })
            .collect()
    }

    pub fn replay_range(&self, start: u64, end: u64) -> Vec<ReplayStep> {
        self.replay()
            .into_iter()
            .filter(|step| step.sequence >= start && step.sequence <= end)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_monotonic_sequence() {
        let mut journal = Journal::new();
        let first = journal.record(10, JournalKind::Observe, "scan", "workspace");
        let second = journal.record(11, JournalKind::Decide, "scope", "ui only");
        assert_eq!(first, 1);
        assert_eq!(second, 2);
        assert_eq!(journal.latest().unwrap().summary, "scope");
    }

    #[test]
    fn replay_uses_relative_offsets() {
        let mut journal = Journal::new();
        journal.record(100, JournalKind::Observe, "scan", "workspace");
        journal.record(104, JournalKind::Plan, "plan", "two steps");
        journal.record(111, JournalKind::Verify, "test", "pass");

        let replay = journal.replay();
        assert_eq!(replay[0].offset, 0);
        assert_eq!(replay[1].offset, 4);
        assert_eq!(replay[2].offset, 11);
    }
}
