// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DecisionStatus {
    Proposed,
    Active,
    Rejected,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitectureDecision {
    pub id: String,
    pub title: String,
    pub rationale: String,
    pub status: DecisionStatus,
    pub scope: BTreeSet<String>,
    pub evidence_ids: BTreeSet<String>,
    pub attempted_approaches: Vec<AttemptedApproach>,
    pub superseded_by: Option<String>,
    pub recorded_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptedApproach {
    pub name: String,
    pub outcome: AttemptOutcome,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptOutcome {
    Adopted,
    Rejected,
    RolledBack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryConflict {
    pub decision_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArchitecturalMemory {
    decisions: BTreeMap<String, ArchitectureDecision>,
}

impl ArchitecturalMemory {
    pub fn record(&mut self, decision: ArchitectureDecision) -> Result<(), &'static str> {
        if decision.id.trim().is_empty() {
            return Err("decision id must not be empty");
        }
        if self.decisions.contains_key(&decision.id) {
            return Err("decision id already exists");
        }
        self.decisions.insert(decision.id.clone(), decision);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&ArchitectureDecision> {
        self.decisions.get(id)
    }

    pub fn active_for_scope<'a>(&'a self, path: &str) -> Vec<&'a ArchitectureDecision> {
        self.decisions
            .values()
            .filter(|decision| decision.status == DecisionStatus::Active)
            .filter(|decision| decision.scope.iter().any(|scope| scope_matches(scope, path)))
            .collect()
    }

    pub fn rejected_approaches_for_scope<'a>(
        &'a self,
        path: &str,
    ) -> Vec<(&'a ArchitectureDecision, &'a AttemptedApproach)> {
        let mut results = Vec::new();
        for decision in self.active_for_scope(path) {
            for attempt in &decision.attempted_approaches {
                if matches!(attempt.outcome, AttemptOutcome::Rejected | AttemptOutcome::RolledBack)
                {
                    results.push((decision, attempt));
                }
            }
        }
        results.sort_by(|left, right| {
            left.0
                .id
                .cmp(&right.0.id)
                .then_with(|| left.1.name.cmp(&right.1.name))
        });
        results
    }

    pub fn conflicts(&self, path: &str, proposed_text: &str) -> Vec<MemoryConflict> {
        let normalized = proposed_text.to_lowercase();
        let mut conflicts = Vec::new();

        for decision in self.active_for_scope(path) {
            for attempt in &decision.attempted_approaches {
                if !matches!(attempt.outcome, AttemptOutcome::Rejected | AttemptOutcome::RolledBack)
                {
                    continue;
                }
                let needle = attempt.name.to_lowercase();
                if !needle.is_empty() && normalized.contains(&needle) {
                    conflicts.push(MemoryConflict {
                        decision_id: decision.id.clone(),
                        reason: format!(
                            "proposal repeats previously {:?} approach: {} ({})",
                            attempt.outcome, attempt.name, attempt.reason
                        ),
                    });
                }
            }
        }

        conflicts.sort_by(|left, right| {
            left.decision_id
                .cmp(&right.decision_id)
                .then_with(|| left.reason.cmp(&right.reason))
        });
        conflicts
    }

    pub fn supersede(&mut self, previous: &str, replacement: &str) -> Result<(), &'static str> {
        if previous == replacement {
            return Err("decision cannot supersede itself");
        }
        if !self.decisions.contains_key(replacement) {
            return Err("replacement decision does not exist");
        }
        let previous = self
            .decisions
            .get_mut(previous)
            .ok_or("previous decision does not exist")?;
        previous.status = DecisionStatus::Superseded;
        previous.superseded_by = Some(replacement.to_owned());
        Ok(())
    }

    pub fn decisions(&self) -> impl Iterator<Item = &ArchitectureDecision> {
        self.decisions.values()
    }
}

fn scope_matches(scope: &str, path: &str) -> bool {
    if scope == "*" {
        return true;
    }
    if let Some(prefix) = scope.strip_suffix("/**") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }
    scope == path
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decision(id: &str, approach: &str, outcome: AttemptOutcome) -> ArchitectureDecision {
        ArchitectureDecision {
            id: id.into(),
            title: "Keep context deterministic".into(),
            rationale: "Avoid hidden model-generated state".into(),
            status: DecisionStatus::Active,
            scope: BTreeSet::from(["src/context/**".into()]),
            evidence_ids: BTreeSet::from(["receipt-42".into()]),
            attempted_approaches: vec![AttemptedApproach {
                name: approach.into(),
                outcome,
                reason: "introduced unstable summaries".into(),
            }],
            superseded_by: None,
            recorded_at_ms: 42,
        }
    }

    #[test]
    fn returns_active_decisions_for_matching_scope() {
        let mut memory = ArchitecturalMemory::default();
        memory
            .record(decision("ADR-1", "LLM summary cache", AttemptOutcome::Rejected))
            .unwrap();
        assert_eq!(memory.active_for_scope("src/context/builder.rs").len(), 1);
        assert!(memory.active_for_scope("src/ui/mod.rs").is_empty());
    }

    #[test]
    fn detects_repeated_rejected_approach() {
        let mut memory = ArchitecturalMemory::default();
        memory
            .record(decision("ADR-1", "LLM summary cache", AttemptOutcome::Rejected))
            .unwrap();
        let conflicts = memory.conflicts(
            "src/context/builder.rs",
            "Add an LLM summary cache before ranking candidates",
        );
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].decision_id, "ADR-1");
    }

    #[test]
    fn supersession_is_explicit_and_deterministic() {
        let mut memory = ArchitecturalMemory::default();
        memory
            .record(decision("ADR-1", "old cache", AttemptOutcome::Rejected))
            .unwrap();
        memory
            .record(decision("ADR-2", "bounded cache", AttemptOutcome::Adopted))
            .unwrap();
        memory.supersede("ADR-1", "ADR-2").unwrap();
        assert_eq!(memory.get("ADR-1").unwrap().status, DecisionStatus::Superseded);
        assert_eq!(memory.get("ADR-1").unwrap().superseded_by.as_deref(), Some("ADR-2"));
    }

    #[test]
    fn ordering_does_not_depend_on_insertion_order() {
        let mut first = ArchitecturalMemory::default();
        first.record(decision("ADR-2", "beta", AttemptOutcome::Rejected)).unwrap();
        first.record(decision("ADR-1", "alpha", AttemptOutcome::Rejected)).unwrap();

        let mut second = ArchitecturalMemory::default();
        second.record(decision("ADR-1", "alpha", AttemptOutcome::Rejected)).unwrap();
        second.record(decision("ADR-2", "beta", AttemptOutcome::Rejected)).unwrap();

        assert_eq!(first, second);
    }
}
