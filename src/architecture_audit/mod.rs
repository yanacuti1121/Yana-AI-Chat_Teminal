// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    Ui,
    Operator,
    Knowledge,
    Gateway,
    Provider,
    Workspace,
    Core,
    Recovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyEdge {
    pub from: Layer,
    pub to: Layer,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryViolation {
    pub edge: DependencyEdge,
    pub rule: String,
}

#[derive(Debug, Clone)]
pub struct ArchitecturePolicy {
    allowed: BTreeMap<Layer, BTreeSet<Layer>>,
}

impl ArchitecturePolicy {
    pub fn yana_default() -> Self {
        let mut allowed = BTreeMap::new();
        allowed.insert(Layer::Ui, BTreeSet::from([Layer::Operator, Layer::Knowledge, Layer::Core, Layer::Recovery]));
        allowed.insert(Layer::Operator, BTreeSet::from([Layer::Knowledge, Layer::Gateway, Layer::Workspace, Layer::Core, Layer::Recovery]));
        allowed.insert(Layer::Knowledge, BTreeSet::from([Layer::Workspace, Layer::Core]));
        allowed.insert(Layer::Gateway, BTreeSet::from([Layer::Provider, Layer::Core]));
        allowed.insert(Layer::Provider, BTreeSet::from([Layer::Gateway]));
        allowed.insert(Layer::Workspace, BTreeSet::from([Layer::Core, Layer::Recovery]));
        allowed.insert(Layer::Core, BTreeSet::new());
        allowed.insert(Layer::Recovery, BTreeSet::from([Layer::Workspace, Layer::Core]));
        Self { allowed }
    }

    pub fn validate(&self, edges: &[DependencyEdge]) -> Vec<BoundaryViolation> {
        edges
            .iter()
            .filter_map(|edge| {
                let allowed = self
                    .allowed
                    .get(&edge.from)
                    .is_some_and(|targets| targets.contains(&edge.to));
                if allowed {
                    None
                } else {
                    Some(BoundaryViolation {
                        edge: edge.clone(),
                        rule: format!("{:?} must not depend on {:?}", edge.from, edge.to),
                    })
                }
            })
            .collect()
    }
}

impl Default for ArchitecturePolicy {
    fn default() -> Self {
        Self::yana_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_provider_workspace_access() {
        let policy = ArchitecturePolicy::default();
        let violations = policy.validate(&[DependencyEdge {
            from: Layer::Provider,
            to: Layer::Workspace,
            reason: "direct file read".into(),
        }]);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn permits_operator_to_workspace_boundary() {
        let policy = ArchitecturePolicy::default();
        let violations = policy.validate(&[DependencyEdge {
            from: Layer::Operator,
            to: Layer::Workspace,
            reason: "approved mutation".into(),
        }]);
        assert!(violations.is_empty());
    }
}
