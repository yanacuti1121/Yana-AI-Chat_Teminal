// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;

use super::{
    budget::{BudgetUsage, ContextBudget},
    estimator::TokenEstimator,
    progressive::ExpansionPlan,
    ranking::{rank_candidates, ContextCandidate},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextSlice {
    pub source: String,
    pub content: String,
    pub score: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBundle {
    pub slices: Vec<ContextSlice>,
    pub usage: BudgetUsage,
    pub considered_candidates: usize,
    pub truncated: bool,
}

pub struct AdaptiveContextBuilder {
    budget: ContextBudget,
    estimator: TokenEstimator,
    expansion: ExpansionPlan,
}

impl AdaptiveContextBuilder {
    pub fn new(budget: ContextBudget) -> Self {
        Self {
            budget,
            estimator: TokenEstimator::default(),
            expansion: ExpansionPlan::default(),
        }
    }

    pub fn with_estimator(mut self, estimator: TokenEstimator) -> Self {
        self.estimator = estimator;
        self
    }

    pub fn with_expansion(mut self, expansion: ExpansionPlan) -> Self {
        self.expansion = expansion;
        self
    }

    pub fn build(&self, mut candidates: Vec<ContextCandidate>) -> ContextBundle {
        rank_candidates(&mut candidates);

        let mut selected = Vec::new();
        let mut seen = BTreeSet::new();
        let mut usage = BudgetUsage::default();
        let mut considered = 0;
        let mut limit = self.expansion.next_limit(0, candidates.len());

        while considered < candidates.len() {
            for candidate in candidates.iter().take(limit).skip(considered) {
                considered += 1;
                if !seen.insert(candidate.source.clone()) {
                    continue;
                }

                let tokens = self.estimator.estimate(&candidate.content);
                let next_usage = BudgetUsage {
                    estimated_tokens: usage.estimated_tokens.saturating_add(tokens),
                    files: usage.files.saturating_add(1),
                    bytes: usage.bytes.saturating_add(candidate.content.len()),
                };

                if !next_usage.fits(self.budget) {
                    continue;
                }

                usage = next_usage;
                selected.push(ContextSlice {
                    source: candidate.source.clone(),
                    content: candidate.content.clone(),
                    score: candidate.score(),
                });
            }

            if usage.files >= self.budget.max_files
                || usage.bytes >= self.budget.max_bytes
                || usage.estimated_tokens >= self.budget.usable_input_tokens()
                || limit >= candidates.len()
            {
                break;
            }

            let next_limit = self.expansion.next_limit(limit, candidates.len());
            if next_limit == limit {
                break;
            }
            limit = next_limit;
        }

        ContextBundle {
            truncated: selected.len() < seen.len() || considered < candidates.len(),
            slices: selected,
            usage,
            considered_candidates: considered,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(source: &str, content: &str, score: u16) -> ContextCandidate {
        ContextCandidate {
            source: source.into(),
            content: content.into(),
            entity_score: score,
            timeline_score: 0,
            receipt_score: 0,
            decision_score: 0,
            workspace_score: 0,
            fact_score: 0,
        }
    }

    #[test]
    fn bundle_never_exceeds_budget() {
        let builder = AdaptiveContextBuilder::new(ContextBudget::new(8, 0, 2, 16));
        let bundle = builder.build(vec![
            candidate("a.rs", "12345678", 3),
            candidate("b.rs", "abcdefgh", 2),
            candidate("c.rs", "ijklmnop", 1),
        ]);
        assert!(bundle.usage.fits(ContextBudget::new(8, 0, 2, 16)));
        assert_eq!(bundle.slices.len(), 2);
    }

    #[test]
    fn duplicate_sources_are_removed() {
        let builder = AdaptiveContextBuilder::new(ContextBudget::standard());
        let bundle = builder.build(vec![
            candidate("a.rs", "first", 2),
            candidate("a.rs", "second", 1),
        ]);
        assert_eq!(bundle.slices.len(), 1);
        assert_eq!(bundle.slices[0].content, "first");
    }

    #[test]
    fn ranking_is_reflected_in_output() {
        let builder = AdaptiveContextBuilder::new(ContextBudget::standard());
        let bundle = builder.build(vec![
            candidate("low.rs", "low", 1),
            candidate("high.rs", "high", 9),
        ]);
        assert_eq!(bundle.slices[0].source, "high.rs");
    }
}
