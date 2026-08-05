// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionStatus {
    Active,
    Superseded { by: String },
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub id: String,
    pub title: String,
    pub rationale: String,
    pub scope: BTreeSet<String>,
    pub status: DecisionStatus,
    pub created_at: u64,
}

#[derive(Debug, Default)]
pub struct DecisionGraph {
    decisions: BTreeMap<String, Decision>,
}

impl DecisionGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, decision: Decision) -> Result<(), DecisionError> {
        if decision.id.trim().is_empty() {
            return Err(DecisionError::EmptyId);
        }
        if self.decisions.contains_key(&decision.id) {
            return Err(DecisionError::DuplicateId(decision.id));
        }
        self.decisions.insert(decision.id.clone(), decision);
        Ok(())
    }

    pub fn active_for(&self, path: &str) -> impl Iterator<Item = &Decision> {
        self.decisions.values().filter(move |decision| {
            decision.status == DecisionStatus::Active
                && decision
                    .scope
                    .iter()
                    .any(|scope| path == scope || path.starts_with(&format!("{scope}/")))
        })
    }

    pub fn supersede(&mut self, id: &str, replacement: &str) -> Result<(), DecisionError> {
        if !self.decisions.contains_key(replacement) {
            return Err(DecisionError::UnknownId(replacement.to_owned()));
        }
        let decision = self
            .decisions
            .get_mut(id)
            .ok_or_else(|| DecisionError::UnknownId(id.to_owned()))?;
        decision.status = DecisionStatus::Superseded {
            by: replacement.to_owned(),
        };
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Decision> {
        self.decisions.get(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionError {
    EmptyId,
    DuplicateId(String),
    UnknownId(String),
}

impl std::fmt::Display for DecisionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyId => write!(formatter, "decision id cannot be empty"),
            Self::DuplicateId(id) => write!(formatter, "decision already exists: {id}"),
            Self::UnknownId(id) => write!(formatter, "unknown decision: {id}"),
        }
    }
}

impl std::error::Error for DecisionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_active_decisions_for_nested_paths() {
        let mut graph = DecisionGraph::new();
        graph
            .record(Decision {
                id: "ui-overlay-only".into(),
                title: "Avoid persistent sidebar".into(),
                rationale: "Terminal-first interaction".into(),
                scope: BTreeSet::from(["src/ui".into()]),
                status: DecisionStatus::Active,
                created_at: 1,
            })
            .unwrap();

        assert_eq!(graph.active_for("src/ui/composer.rs").count(), 1);
        assert_eq!(graph.active_for("src/gateway/mod.rs").count(), 0);
    }
}
