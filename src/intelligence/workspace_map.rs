// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArchitecturalZone {
    Core,
    Stable,
    Experimental,
    Generated,
    ThirdParty,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleSignal {
    pub path: String,
    pub dependencies: BTreeSet<String>,
    pub change_count: u32,
    pub failure_count: u32,
    pub compile_millis: u64,
    pub test_millis: u64,
    pub confidence_percent: u8,
    pub zone: ArchitecturalZone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hotspot {
    pub path: String,
    pub score: u32,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImpactReport {
    pub root: String,
    pub affected: Vec<String>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceMap {
    modules: BTreeMap<String, ModuleSignal>,
    reverse_dependencies: BTreeMap<String, BTreeSet<String>>,
}

impl WorkspaceMap {
    pub fn build(signals: impl IntoIterator<Item = ModuleSignal>) -> Self {
        let mut modules = BTreeMap::new();
        for signal in signals {
            modules.insert(signal.path.clone(), signal);
        }

        let mut reverse_dependencies: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (path, signal) in &modules {
            for dependency in &signal.dependencies {
                reverse_dependencies
                    .entry(dependency.clone())
                    .or_default()
                    .insert(path.clone());
            }
        }

        Self {
            modules,
            reverse_dependencies,
        }
    }

    pub fn module(&self, path: &str) -> Option<&ModuleSignal> {
        self.modules.get(path)
    }

    pub fn modules(&self) -> impl Iterator<Item = &ModuleSignal> {
        self.modules.values()
    }

    pub fn hotspots(&self, limit: usize) -> Vec<Hotspot> {
        let mut hotspots = self
            .modules
            .values()
            .map(|module| {
                let mut reasons = Vec::new();
                let score = module.change_count.saturating_mul(3)
                    + module.failure_count.saturating_mul(8)
                    + ((module.compile_millis / 100).min(u64::from(u32::MAX)) as u32)
                    + ((module.test_millis / 100).min(u64::from(u32::MAX)) as u32);

                if module.change_count >= 10 {
                    reasons.push("frequently changed".into());
                }
                if module.failure_count >= 3 {
                    reasons.push("repeated verification failures".into());
                }
                if module.compile_millis >= 1_000 {
                    reasons.push("slow compile path".into());
                }
                if module.test_millis >= 1_000 {
                    reasons.push("slow test path".into());
                }
                if reasons.is_empty() {
                    reasons.push("combined operational activity".into());
                }

                Hotspot {
                    path: module.path.clone(),
                    score,
                    reasons,
                }
            })
            .collect::<Vec<_>>();

        hotspots.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.path.cmp(&right.path))
        });
        hotspots.truncate(limit);
        hotspots
    }

    pub fn impact(&self, root: &str, max_depth: usize, max_nodes: usize) -> ImpactReport {
        let mut queue = VecDeque::from([(root.to_owned(), 0usize)]);
        let mut visited = BTreeSet::from([root.to_owned()]);
        let mut affected = Vec::new();
        let mut truncated = false;

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            let Some(dependents) = self.reverse_dependencies.get(&current) else {
                continue;
            };
            for dependent in dependents {
                if !visited.insert(dependent.clone()) {
                    continue;
                }
                if affected.len() >= max_nodes {
                    truncated = true;
                    break;
                }
                affected.push(dependent.clone());
                queue.push_back((dependent.clone(), depth + 1));
            }
            if truncated {
                break;
            }
        }

        ImpactReport {
            root: root.to_owned(),
            affected,
            truncated,
        }
    }

    pub fn mutation_allowed_by_zone(&self, path: &str) -> bool {
        self.modules.get(path).is_none_or(|module| {
            !matches!(
                module.zone,
                ArchitecturalZone::Generated | ArchitecturalZone::ThirdParty
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signal(path: &str, dependencies: &[&str], zone: ArchitecturalZone) -> ModuleSignal {
        ModuleSignal {
            path: path.into(),
            dependencies: dependencies.iter().map(|value| (*value).into()).collect(),
            change_count: 0,
            failure_count: 0,
            compile_millis: 0,
            test_millis: 0,
            confidence_percent: 90,
            zone,
        }
    }

    #[test]
    fn impact_follows_reverse_dependencies_deterministically() {
        let map = WorkspaceMap::build([
            signal("context", &[], ArchitecturalZone::Core),
            signal("gateway", &["context"], ArchitecturalZone::Stable),
            signal("ui", &["gateway"], ArchitecturalZone::Stable),
        ]);
        let report = map.impact("context", 3, 10);
        assert_eq!(report.affected, vec!["gateway", "ui"]);
        assert!(!report.truncated);
    }

    #[test]
    fn generated_and_third_party_zones_are_not_mutation_targets() {
        let map = WorkspaceMap::build([
            signal("generated", &[], ArchitecturalZone::Generated),
            signal("vendor", &[], ArchitecturalZone::ThirdParty),
        ]);
        assert!(!map.mutation_allowed_by_zone("generated"));
        assert!(!map.mutation_allowed_by_zone("vendor"));
    }

    #[test]
    fn hotspot_order_is_stable() {
        let mut a = signal("a", &[], ArchitecturalZone::Stable);
        a.change_count = 20;
        let mut b = signal("b", &[], ArchitecturalZone::Stable);
        b.failure_count = 8;
        let map = WorkspaceMap::build([a, b]);
        assert_eq!(map.hotspots(2)[0].path, "b");
    }
}
