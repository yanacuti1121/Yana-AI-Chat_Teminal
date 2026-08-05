// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimelineKind {
    Observe,
    Plan,
    Execute,
    Verify,
    Reflect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: u64,
    pub session_id: String,
    pub task_id: String,
    pub timestamp: u64,
    pub kind: TimelineKind,
    pub summary: String,
    pub references: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TimelineTree {
    next_id: u64,
    events: Vec<TimelineEvent>,
    by_session: BTreeMap<String, Vec<u64>>,
    by_task: BTreeMap<String, Vec<u64>>,
}

impl TimelineTree {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            ..Self::default()
        }
    }

    pub fn record(
        &mut self,
        session_id: impl Into<String>,
        task_id: impl Into<String>,
        timestamp: u64,
        kind: TimelineKind,
        summary: impl Into<String>,
        references: Vec<String>,
    ) -> u64 {
        let session_id = session_id.into();
        let task_id = task_id.into();
        let id = self.next_id;
        self.next_id += 1;

        self.events.push(TimelineEvent {
            id,
            session_id: session_id.clone(),
            task_id: task_id.clone(),
            timestamp,
            kind,
            summary: summary.into(),
            references,
        });
        self.by_session.entry(session_id).or_default().push(id);
        self.by_task.entry(task_id).or_default().push(id);
        id
    }

    pub fn task_events(&self, task_id: &str) -> impl Iterator<Item = &TimelineEvent> {
        self.by_task
            .get(task_id)
            .into_iter()
            .flatten()
            .filter_map(|id| self.events.iter().find(|event| event.id == *id))
    }

    pub fn session_events(&self, session_id: &str) -> impl Iterator<Item = &TimelineEvent> {
        self.by_session
            .get(session_id)
            .into_iter()
            .flatten()
            .filter_map(|id| self.events.iter().find(|event| event.id == *id))
    }

    pub fn recent(&self, limit: usize) -> impl Iterator<Item = &TimelineEvent> {
        self.events.iter().rev().take(limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_task_order_without_llm_summaries() {
        let mut timeline = TimelineTree::new();
        timeline.record("s1", "t1", 10, TimelineKind::Observe, "read ui", vec![]);
        timeline.record("s1", "t1", 20, TimelineKind::Verify, "tests pass", vec![]);

        let kinds = timeline
            .task_events("t1")
            .map(|event| event.kind)
            .collect::<Vec<_>>();
        assert_eq!(kinds, vec![TimelineKind::Observe, TimelineKind::Verify]);
    }
}
