// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentKind {
    Terminal,
    Desktop,
    VsCode,
    JetBrains,
    ClaudeCode,
    Codex,
    Cursor,
    Antigravity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeEndpoint {
    pub id: String,
    pub kind: EnvironmentKind,
    pub protocol_version: u16,
    pub connected: bool,
}

#[derive(Debug, Default)]
pub struct Bridge {
    endpoints: BTreeMap<String, BridgeEndpoint>,
}

impl Bridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, endpoint: BridgeEndpoint) -> Result<(), BridgeError> {
        if endpoint.id.trim().is_empty() {
            return Err(BridgeError::InvalidEndpointId);
        }
        if self.endpoints.contains_key(&endpoint.id) {
            return Err(BridgeError::DuplicateEndpoint(endpoint.id));
        }
        self.endpoints.insert(endpoint.id.clone(), endpoint);
        Ok(())
    }

    pub fn set_connected(&mut self, id: &str, connected: bool) -> Result<(), BridgeError> {
        let endpoint = self
            .endpoints
            .get_mut(id)
            .ok_or_else(|| BridgeError::UnknownEndpoint(id.to_owned()))?;
        endpoint.connected = connected;
        Ok(())
    }

    pub fn endpoint(&self, id: &str) -> Option<&BridgeEndpoint> {
        self.endpoints.get(id)
    }

    pub fn connected(&self) -> impl Iterator<Item = &BridgeEndpoint> {
        self.endpoints.values().filter(|endpoint| endpoint.connected)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeError {
    InvalidEndpointId,
    DuplicateEndpoint(String),
    UnknownEndpoint(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidEndpointId => write!(formatter, "bridge endpoint id cannot be empty"),
            Self::DuplicateEndpoint(id) => write!(formatter, "bridge endpoint exists: {id}"),
            Self::UnknownEndpoint(id) => write!(formatter, "unknown bridge endpoint: {id}"),
        }
    }
}

impl std::error::Error for BridgeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_connected_environments() {
        let mut bridge = Bridge::new();
        bridge
            .register(BridgeEndpoint {
                id: "terminal".into(),
                kind: EnvironmentKind::Terminal,
                protocol_version: 1,
                connected: false,
            })
            .unwrap();
        bridge.set_connected("terminal", true).unwrap();
        assert_eq!(bridge.connected().count(), 1);
    }
}
