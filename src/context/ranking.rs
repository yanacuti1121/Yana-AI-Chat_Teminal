// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextCandidate {
    pub source: String,
    pub content: String,
    pub entity_score: u16,
    pub timeline_score: u16,
    pub receipt_score: u16,
    pub decision_score: u16,
    pub workspace_score: u16,
    pub fact_score: u16,
}

impl ContextCandidate {
    pub fn score(&self) -> u32 {
        u32::from(self.entity_score) * 5
            + u32::from(self.decision_score) * 5
            + u32::from(self.fact_score) * 4
            + u32::from(self.workspace_score) * 3
            + u32::from(self.receipt_score) * 2
            + u32::from(self.timeline_score)
    }
}

pub fn rank_candidates(candidates: &mut [ContextCandidate]) {
    candidates.sort_by(|left, right| {
        right
            .score()
            .cmp(&left.score())
            .then_with(|| left.source.cmp(&right.source))
            .then(Ordering::Equal)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(source: &str, entity: u16, decision: u16) -> ContextCandidate {
        ContextCandidate {
            source: source.into(),
            content: source.into(),
            entity_score: entity,
            timeline_score: 0,
            receipt_score: 0,
            decision_score: decision,
            workspace_score: 0,
            fact_score: 0,
        }
    }

    #[test]
    fn ranking_is_stable_for_equal_scores() {
        let mut values = vec![candidate("b.rs", 1, 0), candidate("a.rs", 1, 0)];
        rank_candidates(&mut values);
        assert_eq!(values[0].source, "a.rs");
        assert_eq!(values[1].source, "b.rs");
    }

    #[test]
    fn decisions_have_high_weight() {
        let mut values = vec![candidate("entity.rs", 2, 0), candidate("decision.rs", 0, 3)];
        rank_candidates(&mut values);
        assert_eq!(values[0].source, "decision.rs");
    }
}
