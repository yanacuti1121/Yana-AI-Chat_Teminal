// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

pub mod entity;
pub mod evidence;
pub mod receipt;
pub mod timeline;

use entity::{EntityGraph, EntityId};
use evidence::EvidenceIndex;
use receipt::ReceiptStore;
use timeline::TimelineTree;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeHit {
    pub source: String,
    pub detail: String,
    pub score: i32,
}

#[derive(Debug, Default)]
pub struct KnowledgeEngine {
    pub timeline: TimelineTree,
    pub entities: EntityGraph,
    pub evidence: EvidenceIndex,
    pub receipts: ReceiptStore,
}

impl KnowledgeEngine {
    pub fn new() -> Self {
        Self {
            timeline: TimelineTree::new(),
            entities: EntityGraph::new(),
            evidence: EvidenceIndex::new(),
            receipts: ReceiptStore::new(),
        }
    }

    pub fn retrieve_for_entity(
        &self,
        entity: &EntityId,
        max_depth: usize,
        limit: usize,
    ) -> Vec<KnowledgeHit> {
        let mut hits = Vec::new();

        for (depth, related) in self.entities.traverse(entity, max_depth) {
            hits.push(KnowledgeHit {
                source: format!("entity:{:?}", related.kind),
                detail: related.value,
                score: 100 - (depth as i32 * 15),
            });
        }

        if entity.kind == entity::EntityKind::File {
            if let Some(evidence) = self.evidence.strongest_for_path(&entity.value) {
                hits.push(KnowledgeHit {
                    source: "evidence".into(),
                    detail: format!(
                        "{}:{}-{} — {}",
                        evidence.path, evidence.start_line, evidence.end_line, evidence.reason
                    ),
                    score: 60 + evidence.confidence as i32 / 2,
                });
            }

            for receipt in self.receipts.for_file(&entity.value) {
                hits.push(KnowledgeHit {
                    source: "receipt".into(),
                    detail: format!("task {} — {}", receipt.task_id, receipt.reason),
                    score: match receipt.outcome {
                        receipt::ReceiptOutcome::Succeeded => 78,
                        receipt::ReceiptOutcome::Failed => 72,
                        receipt::ReceiptOutcome::Rejected => 64,
                        receipt::ReceiptOutcome::Cancelled => 58,
                    },
                });
            }
        }

        hits.sort_by(|left, right| right.score.cmp(&left.score).then_with(|| left.detail.cmp(&right.detail)));
        hits.dedup_by(|left, right| left.source == right.source && left.detail == right.detail);
        hits.truncate(limit);
        hits
    }

    pub fn build_context(&self, hits: &[KnowledgeHit], max_chars: usize) -> String {
        let mut output = String::new();
        for hit in hits {
            let line = format!("- [{} | {}] {}\n", hit.source, hit.score, hit.detail);
            if output.len() + line.len() > max_chars {
                break;
            }
            output.push_str(&line);
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use entity::{EntityKind, Relation};
    use receipt::{ReceiptOutcome, WorkspaceReceipt};

    #[test]
    fn retrieval_combines_graph_evidence_and_receipts_without_model_calls() {
        let file = EntityId::new(EntityKind::File, "src/ui/header.rs");
        let symbol = EntityId::new(EntityKind::Symbol, "Header");
        let mut engine = KnowledgeEngine::new();
        engine
            .entities
            .connect(file.clone(), Relation::Contains, symbol);
        engine
            .evidence
            .record_evidence(
                "src/ui/header.rs",
                1,
                20,
                "defines the terminal header",
                10,
                "atlas",
                95,
            )
            .unwrap();
        engine.receipts.append(WorkspaceReceipt {
            id: 0,
            task_id: "task-header".into(),
            timestamp: 11,
            duration_ms: 5,
            files: vec!["src/ui/header.rs".into()],
            decision_ids: vec![],
            evidence_ids: vec![1],
            tests: vec![],
            reason: "apply branded header".into(),
            outcome: ReceiptOutcome::Succeeded,
        });

        let hits = engine.retrieve_for_entity(&file, 1, 8);
        assert!(hits.iter().any(|hit| hit.source == "evidence"));
        assert!(hits.iter().any(|hit| hit.source == "receipt"));
        assert!(engine.build_context(&hits, 500).contains("Header"));
    }
}
