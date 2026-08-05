// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::model::{ModelError, ModelRequest, ModelResponse, StreamEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdapterKind {
    OpenAiCompatible,
    AnthropicCompatible,
    Ollama,
    LlamaCpp,
    Mlx,
    Embedded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterConfig {
    pub id: String,
    pub kind: AdapterKind,
    pub endpoint: String,
    pub api_key_env: Option<String>,
    pub timeout_ms: u64,
    pub headers: BTreeMap<String, String>,
}

impl AdapterConfig {
    pub fn validate(&self) -> Result<(), ModelError> {
        if self.id.trim().is_empty() {
            return Err(ModelError::InvalidRequest("adapter id cannot be empty".into()));
        }
        if self.timeout_ms == 0 {
            return Err(ModelError::InvalidRequest(
                "adapter timeout must be greater than zero".into(),
            ));
        }
        if !matches!(self.kind, AdapterKind::Embedded)
            && !(self.endpoint.starts_with("http://") || self.endpoint.starts_with("https://"))
        {
            return Err(ModelError::InvalidRequest(
                "adapter endpoint must use http or https".into(),
            ));
        }
        Ok(())
    }
}

pub trait ProviderAdapter {
    fn id(&self) -> &str;
    fn config(&self) -> &AdapterConfig;
    fn discover_models(&self) -> Result<Vec<ModelDescriptor>, ModelError>;
    fn complete(&self, request: &ModelRequest) -> Result<ModelResponse, ModelError>;
    fn stream(
        &self,
        request: &ModelRequest,
        emit: &mut dyn FnMut(StreamEvent) -> Result<(), ModelError>,
    ) -> Result<(), ModelError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelDescriptor {
    pub id: String,
    pub display_name: String,
    pub context_window: u32,
    pub capabilities: Vec<ModelCapability>,
    pub local: bool,
}

impl ModelDescriptor {
    pub fn supports(&self, capability: ModelCapability) -> bool {
        self.capabilities.contains(&capability)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelCapability {
    Chat,
    Streaming,
    ToolCalling,
    Vision,
    Reasoning,
    Embeddings,
}

#[derive(Default)]
pub struct AdapterRegistry {
    adapters: BTreeMap<String, Box<dyn ProviderAdapter>>,
}

impl AdapterRegistry {
    pub fn register(&mut self, adapter: Box<dyn ProviderAdapter>) -> Result<(), ModelError> {
        let id = adapter.id().to_owned();
        adapter.config().validate()?;
        if self.adapters.contains_key(&id) {
            return Err(ModelError::InvalidRequest(format!(
                "adapter already registered: {id}"
            )));
        }
        self.adapters.insert(id, adapter);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&dyn ProviderAdapter> {
        self.adapters.get(id).map(Box::as_ref)
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.adapters.keys().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_http_remote_endpoint() {
        let config = AdapterConfig {
            id: "ollama".into(),
            kind: AdapterKind::Ollama,
            endpoint: "localhost:11434".into(),
            api_key_env: None,
            timeout_ms: 30_000,
            headers: BTreeMap::new(),
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn model_capability_lookup_is_explicit() {
        let model = ModelDescriptor {
            id: "gemma".into(),
            display_name: "Gemma".into(),
            context_window: 8192,
            capabilities: vec![ModelCapability::Chat, ModelCapability::Streaming],
            local: true,
        };
        assert!(model.supports(ModelCapability::Streaming));
        assert!(!model.supports(ModelCapability::Vision));
    }
}
