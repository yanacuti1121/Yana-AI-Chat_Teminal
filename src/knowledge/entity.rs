// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EntityKind {
    Workspace,
    File,
    Module,
    Symbol,
    Decision,
    Test,
    Commit,
    Receipt,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EntityId {
    pub kind: EntityKind,
    pub value: String,
}

impl EntityId {
    pub fn new(kind: EntityKind, value: impl Into<String>) -> Self {
        Self {
            kind,
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Relation {
    Contains,
    Imports,
    Calls,
    Tests,
    Implements,
    BelongsTo,
    DecidedBy,
    ProducedBy,
    Verifies,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Edge {
    pub from: EntityId,
    pub relation: Relation,
    pub to: EntityId,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EntityGraph {
    nodes: BTreeSet<EntityId>,
    outgoing: BTreeMap<EntityId, BTreeSet<Edge>>,
}

impl EntityGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn connect(&mut self, from: EntityId, relation: Relation, to: EntityId) {
        self.nodes.insert(from.clone());
        self.nodes.insert(to.clone());
        self.outgoing.entry(from.clone()).or_default().insert(Edge {
            from,
            relation,
            to,
        });
    }

    pub fn neighbors(&self, entity: &EntityId) -> impl Iterator<Item = &Edge> {
        self.outgoing.get(entity).into_iter().flatten()
    }

    pub fn traverse(&self, start: &EntityId, max_depth: usize) -> Vec<(usize, EntityId)> {
        let mut queue = VecDeque::from([(0usize, start.clone())]);
        let mut visited = BTreeSet::new();
        let mut result = Vec::new();

        while let Some((depth, current)) = queue.pop_front() {
            if depth > max_depth || !visited.insert(current.clone()) {
                continue;
            }
            result.push((depth, current.clone()));
            if depth == max_depth {
                continue;
            }
            for edge in self.neighbors(&current) {
                queue.push_back((depth + 1, edge.to.clone()));
            }
        }
        result
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traverses_related_code_entities_with_a_hard_depth_limit() {
        let file = EntityId::new(EntityKind::File, "src/ui/header.rs");
        let symbol = EntityId::new(EntityKind::Symbol, "Header");
        let test = EntityId::new(EntityKind::Test, "header_render");
        let mut graph = EntityGraph::new();
        graph.connect(file.clone(), Relation::Contains, symbol.clone());
        graph.connect(symbol, Relation::Tests, test);

        let result = graph.traverse(&file, 1);
        assert_eq!(result.len(), 2);
    }
}
