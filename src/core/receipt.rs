// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreReceipt {
    pub task_id: String,
    pub action: String,
    pub outcome: String,
    pub files: Vec<PathBuf>,
    pub evidence_ids: Vec<String>,
    pub decision_ids: Vec<String>,
    pub timestamp_ms: u64,
}

pub trait ReceiptSink {
    fn append(&mut self, receipt: &CoreReceipt) -> Result<(), ReceiptError>;
}

#[derive(Debug, Default)]
pub struct MemoryReceiptSink {
    receipts: Vec<CoreReceipt>,
}

impl MemoryReceiptSink {
    pub fn receipts(&self) -> &[CoreReceipt] {
        &self.receipts
    }
}

impl ReceiptSink for MemoryReceiptSink {
    fn append(&mut self, receipt: &CoreReceipt) -> Result<(), ReceiptError> {
        self.receipts.push(receipt.clone());
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiptError {
    Rejected(String),
}

impl std::fmt::Display for ReceiptError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rejected(message) => write!(formatter, "receipt rejected: {message}"),
        }
    }
}

impl std::error::Error for ReceiptError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sink_keeps_receipts_append_only() {
        let mut sink = MemoryReceiptSink::default();
        let receipt = CoreReceipt {
            task_id: "task-1".into(),
            action: "patch".into(),
            outcome: "approved".into(),
            files: vec!["src/main.rs".into()],
            evidence_ids: vec!["ev-1".into()],
            decision_ids: Vec::new(),
            timestamp_ms: 1,
        };
        sink.append(&receipt).unwrap();
        assert_eq!(sink.receipts(), &[receipt]);
    }
}
