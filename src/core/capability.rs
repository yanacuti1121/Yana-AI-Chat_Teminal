// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::{collections::BTreeMap, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Capability {
    Guard,
    Doctor,
    Audit,
    Sandbox,
    Skills,
    TokenBudget,
    Halt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityState {
    Available,
    Missing,
}

#[derive(Debug, Clone, Default)]
pub struct CapabilityRegistry {
    states: BTreeMap<Capability, CapabilityState>,
}

impl CapabilityRegistry {
    pub fn detect(core_root: &Path) -> Self {
        let mut registry = Self::default();
        registry.set(Capability::Guard, any_exists(core_root, &["core/hooks/guard-destructive.sh", "src/guards"]));
        registry.set(Capability::Doctor, any_exists(core_root, &["src/doctor", "bin/yana"]));
        registry.set(Capability::Audit, any_exists(core_root, &["core/scripts/audit-log.sh", ".claude/state/audit-chain.log"]));
        registry.set(Capability::Sandbox, any_exists(core_root, &["core/scripts/sandbox-exec.sh", "scripts/sandbox-exec.sh"]));
        registry.set(Capability::Skills, any_exists(core_root, &["core/skills", ".claude/skills"]));
        registry.set(Capability::TokenBudget, any_exists(core_root, &["core/hooks/token-budget.sh", "src/guards/token_budget.rs"]));
        registry.set(Capability::Halt, any_exists(core_root, &["core/hooks/giamthi-halt-check.sh", ".claude/hooks/giamthi-halt-check.sh"]));
        registry
    }

    pub fn state(&self, capability: Capability) -> CapabilityState {
        self.states
            .get(&capability)
            .copied()
            .unwrap_or(CapabilityState::Missing)
    }

    pub fn available(&self, capability: Capability) -> bool {
        self.state(capability) == CapabilityState::Available
    }

    pub fn iter(&self) -> impl Iterator<Item = (Capability, CapabilityState)> + '_ {
        self.states.iter().map(|(capability, state)| (*capability, *state))
    }

    fn set(&mut self, capability: Capability, available: bool) {
        self.states.insert(
            capability,
            if available {
                CapabilityState::Available
            } else {
                CapabilityState::Missing
            },
        );
    }
}

fn any_exists(root: &Path, candidates: &[&str]) -> bool {
    candidates.iter().any(|candidate| root.join(candidate).exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_capability_defaults_to_missing() {
        let registry = CapabilityRegistry::default();
        assert_eq!(registry.state(Capability::Guard), CapabilityState::Missing);
    }
}
