// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

mod ollama;
mod openai;

use std::collections::BTreeMap;

use crate::{
    adapters::{AdapterConfig, AdapterKind, AdapterRegistry},
    model::ModelError,
};

pub use ollama::OllamaAdapter;
pub use openai::OpenAiCompatibleAdapter;

pub fn ollama_config(endpoint: impl Into<String>) -> AdapterConfig {
    AdapterConfig {
        id: "ollama".into(),
        kind: AdapterKind::Ollama,
        endpoint: endpoint.into(),
        api_key_env: None,
        timeout_ms: 120_000,
        headers: BTreeMap::new(),
    }
}

pub fn lm_studio_config(endpoint: impl Into<String>) -> AdapterConfig {
    AdapterConfig {
        id: "lm-studio".into(),
        kind: AdapterKind::OpenAiCompatible,
        endpoint: endpoint.into(),
        api_key_env: None,
        timeout_ms: 120_000,
        headers: BTreeMap::new(),
    }
}

pub fn openai_config(endpoint: impl Into<String>, api_key_env: impl Into<String>) -> AdapterConfig {
    AdapterConfig {
        id: "openai".into(),
        kind: AdapterKind::OpenAiCompatible,
        endpoint: endpoint.into(),
        api_key_env: Some(api_key_env.into()),
        timeout_ms: 120_000,
        headers: BTreeMap::new(),
    }
}

pub fn register_local_defaults(registry: &mut AdapterRegistry) -> Result<(), ModelError> {
    registry.register(Box::new(OllamaAdapter::new(ollama_config(
        "http://127.0.0.1:11434",
    ))?))?;
    registry.register(Box::new(OpenAiCompatibleAdapter::new(lm_studio_config(
        "http://127.0.0.1:1234",
    ))?))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_presets_do_not_persist_api_keys() {
        assert!(ollama_config("http://localhost:11434").api_key_env.is_none());
        assert!(lm_studio_config("http://localhost:1234").api_key_env.is_none());
    }
}
