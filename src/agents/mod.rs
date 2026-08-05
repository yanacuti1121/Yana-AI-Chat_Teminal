// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AgentCapability {
    Plan,
    Research,
    Build,
    Review,
    Verify,
    IndexWorkspace,
    QueryKnowledge,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AgentPermission {
    ReadWorkspace,
    ProposeMutation,
    RequestVerification,
    RequestProvider,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentManifest {
    pub id: String,
    pub version: String,
    pub display_name: String,
    pub capabilities: BTreeSet<AgentCapability>,
    pub permissions: BTreeSet<AgentPermission>,
    pub required_services: BTreeSet<String>,
}

impl AgentManifest {
    pub fn validate(&self) -> Result<(), RegistryError> {
        if !valid_identifier(&self.id) {
            return Err(RegistryError::InvalidId(self.id.clone()));
        }
        if !valid_version(&self.version) {
            return Err(RegistryError::InvalidVersion(self.version.clone()));
        }
        if self.display_name.trim().is_empty() {
            return Err(RegistryError::EmptyDisplayName);
        }
        if self.capabilities.is_empty() {
            return Err(RegistryError::MissingCapabilities);
        }
        if self.permissions.contains(&AgentPermission::ProposeMutation)
            && !self.permissions.contains(&AgentPermission::ReadWorkspace)
        {
            return Err(RegistryError::MutationWithoutReadPermission);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct AgentRegistry {
    agents: BTreeMap<String, AgentManifest>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, manifest: AgentManifest) -> Result<(), RegistryError> {
        manifest.validate()?;
        if self.agents.contains_key(&manifest.id) {
            return Err(RegistryError::DuplicateAgent(manifest.id));
        }
        self.agents.insert(manifest.id.clone(), manifest);
        Ok(())
    }

    pub fn replace(&mut self, manifest: AgentManifest) -> Result<Option<AgentManifest>, RegistryError> {
        manifest.validate()?;
        Ok(self.agents.insert(manifest.id.clone(), manifest))
    }

    pub fn remove(&mut self, id: &str) -> Option<AgentManifest> {
        self.agents.remove(id)
    }

    pub fn get(&self, id: &str) -> Option<&AgentManifest> {
        self.agents.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &AgentManifest> {
        self.agents.values()
    }

    pub fn matching(&self, required: &BTreeSet<AgentCapability>) -> Vec<&AgentManifest> {
        self.agents
            .values()
            .filter(|agent| required.is_subset(&agent.capabilities))
            .collect()
    }

    pub fn authorized(&self, id: &str, permission: &AgentPermission) -> bool {
        self.get(id)
            .is_some_and(|agent| agent.permissions.contains(permission))
    }

    pub fn len(&self) -> usize {
        self.agents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_version(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    InvalidId(String),
    InvalidVersion(String),
    EmptyDisplayName,
    MissingCapabilities,
    MutationWithoutReadPermission,
    DuplicateAgent(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidId(id) => write!(formatter, "invalid agent id: {id}"),
            Self::InvalidVersion(version) => write!(formatter, "invalid agent version: {version}"),
            Self::EmptyDisplayName => write!(formatter, "agent display name is empty"),
            Self::MissingCapabilities => write!(formatter, "agent has no capabilities"),
            Self::MutationWithoutReadPermission => {
                write!(formatter, "mutation proposal requires workspace read permission")
            }
            Self::DuplicateAgent(id) => write!(formatter, "agent is already registered: {id}"),
        }
    }
}

impl std::error::Error for RegistryError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(id: &str, capabilities: &[AgentCapability]) -> AgentManifest {
        AgentManifest {
            id: id.into(),
            version: "1.0.0".into(),
            display_name: id.into(),
            capabilities: capabilities.iter().cloned().collect(),
            permissions: [AgentPermission::ReadWorkspace].into_iter().collect(),
            required_services: BTreeSet::new(),
        }
    }

    #[test]
    fn registration_and_iteration_are_deterministic() {
        let mut registry = AgentRegistry::new();
        registry.register(manifest("reviewer", &[AgentCapability::Review])).unwrap();
        registry.register(manifest("builder", &[AgentCapability::Build])).unwrap();

        let ids = registry.iter().map(|agent| agent.id.as_str()).collect::<Vec<_>>();
        assert_eq!(ids, vec!["builder", "reviewer"]);
    }

    #[test]
    fn rejects_duplicate_agents() {
        let mut registry = AgentRegistry::new();
        registry.register(manifest("builder", &[AgentCapability::Build])).unwrap();
        assert_eq!(
            registry.register(manifest("builder", &[AgentCapability::Build])),
            Err(RegistryError::DuplicateAgent("builder".into()))
        );
    }

    #[test]
    fn finds_agents_with_all_required_capabilities() {
        let mut registry = AgentRegistry::new();
        registry
            .register(manifest(
                "builder",
                &[AgentCapability::Build, AgentCapability::Verify],
            ))
            .unwrap();
        registry.register(manifest("reviewer", &[AgentCapability::Review])).unwrap();

        let required = [AgentCapability::Build, AgentCapability::Verify]
            .into_iter()
            .collect();
        let matches = registry.matching(&required);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "builder");
    }

    #[test]
    fn mutation_permission_requires_read_permission() {
        let mut candidate = manifest("builder", &[AgentCapability::Build]);
        candidate.permissions = [AgentPermission::ProposeMutation].into_iter().collect();
        assert_eq!(
            candidate.validate(),
            Err(RegistryError::MutationWithoutReadPermission)
        );
    }
}
