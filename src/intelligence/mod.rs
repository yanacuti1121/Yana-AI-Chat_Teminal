// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub path: String,
    pub symbol_match: u8,
    pub dependency_relevance: u8,
    pub history_relevance: u8,
    pub test_relevance: u8,
    pub decision_conflict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedCandidate {
    pub path: String,
    pub confidence: u8,
    pub reasons: Vec<String>,
    pub decision_conflict: bool,
}

#[derive(Debug, Default)]
pub struct WorkspaceIntelligence;

impl WorkspaceIntelligence {
    pub fn rank(candidates: impl IntoIterator<Item = Candidate>) -> Vec<RankedCandidate> {
        let mut ranked = candidates
            .into_iter()
            .map(|candidate| {
                let weighted = u16::from(candidate.symbol_match) * 4
                    + u16::from(candidate.dependency_relevance) * 3
                    + u16::from(candidate.history_relevance) * 2
                    + u16::from(candidate.test_relevance);
                let confidence = (weighted / 10).min(100) as u8;
                let mut reasons = Vec::new();

                if candidate.symbol_match >= 70 {
                    reasons.push("strong symbol match".into());
                }
                if candidate.dependency_relevance >= 70 {
                    reasons.push("high dependency relevance".into());
                }
                if candidate.history_relevance >= 70 {
                    reasons.push("similar files changed in prior tasks".into());
                }
                if candidate.test_relevance >= 70 {
                    reasons.push("directly covered by related tests".into());
                }
                if candidate.decision_conflict {
                    reasons.push("conflicts with an active architecture decision".into());
                }
                if reasons.is_empty() {
                    reasons.push("weak combined workspace signal".into());
                }

                RankedCandidate {
                    path: candidate.path,
                    confidence,
                    reasons,
                    decision_conflict: candidate.decision_conflict,
                }
            })
            .collect::<Vec<_>>();

        ranked.sort_by(|left, right| {
            left.decision_conflict
                .cmp(&right.decision_conflict)
                .then_with(|| right.confidence.cmp(&left.confidence))
                .then_with(|| left.path.cmp(&right.path))
        });
        ranked
    }

    pub fn recommended_scope(
        candidates: &[RankedCandidate],
        minimum_confidence: u8,
        limit: usize,
    ) -> Vec<&RankedCandidate> {
        candidates
            .iter()
            .filter(|candidate| {
                candidate.confidence >= minimum_confidence && !candidate.decision_conflict
            })
            .take(limit)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_decision_conflicts_from_recommended_scope() {
        let ranked = WorkspaceIntelligence::rank([
            Candidate {
                path: "src/ui/composer.rs".into(),
                symbol_match: 95,
                dependency_relevance: 90,
                history_relevance: 70,
                test_relevance: 80,
                decision_conflict: false,
            },
            Candidate {
                path: "src/ui/sidebar.rs".into(),
                symbol_match: 99,
                dependency_relevance: 90,
                history_relevance: 90,
                test_relevance: 20,
                decision_conflict: true,
            },
        ]);

        let scope = WorkspaceIntelligence::recommended_scope(&ranked, 60, 5);
        assert_eq!(scope.len(), 1);
        assert_eq!(scope[0].path, "src/ui/composer.rs");
    }
}
