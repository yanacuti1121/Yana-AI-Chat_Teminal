// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReceiptOutcome {
    Succeeded,
    Failed,
    Rejected,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceReceipt {
    pub id: u64,
    pub task_id: String,
    pub timestamp: u64,
    pub duration_ms: u64,
    pub files: Vec<String>,
    pub decision_ids: Vec<String>,
    pub evidence_ids: Vec<u64>,
    pub tests: Vec<String>,
    pub reason: String,
    pub outcome: ReceiptOutcome,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReceiptStore {
    next_id: u64,
    receipts: Vec<WorkspaceReceipt>,
}

impl ReceiptStore {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            ..Self::default()
        }
    }

    pub fn append(&mut self, mut receipt: WorkspaceReceipt) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        receipt.id = id;
        self.receipts.push(receipt);
        id
    }

    pub fn get(&self, id: u64) -> Option<&WorkspaceReceipt> {
        self.receipts.iter().find(|receipt| receipt.id == id)
    }

    pub fn for_file<'a>(&'a self, path: &'a str) -> impl Iterator<Item = &'a WorkspaceReceipt> {
        self.receipts
            .iter()
            .filter(move |receipt| receipt.files.iter().any(|file| file == path))
    }

    pub fn for_task<'a>(&'a self, task_id: &'a str) -> impl Iterator<Item = &'a WorkspaceReceipt> {
        self.receipts
            .iter()
            .filter(move |receipt| receipt.task_id == task_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipts_are_append_only_and_queryable_by_file() {
        let mut store = ReceiptStore::new();
        let id = store.append(WorkspaceReceipt {
            id: 0,
            task_id: "task-1".into(),
            timestamp: 10,
            duration_ms: 4,
            files: vec!["src/ui/header.rs".into()],
            decision_ids: vec!["decision-7".into()],
            evidence_ids: vec![3],
            tests: vec!["header_render".into()],
            reason: "apply Sky Lake identity".into(),
            outcome: ReceiptOutcome::Succeeded,
        });

        assert_eq!(id, 1);
        assert_eq!(store.for_file("src/ui/header.rs").count(), 1);
    }
}
