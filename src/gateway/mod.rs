// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Capability {
    Chat,
    Streaming,
    ToolCalling,
    Vision,
    Embeddings,
    Reasoning,
    ImageGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderManifest {
    pub id: String,
    pub display_name: String,
    pub transport: Transport,
    pub local: bool,
    pub capabilities: Vec<Capability>,
    pub default_model: Option<String>,
}

impl ProviderManifest {
    pub fn supports(&self, capability: Capability) -> bool {
        self.capabilities.contains(&capability)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    OpenAiCompatible,
    AnthropicCompatible,
    Ollama,
    LlamaCpp,
    Mlx,
    Embedded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteRequest {
    pub required: Vec<Capability>,
    pub prefer_local: bool,
    pub preferred_provider: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteDecision {
    pub provider_id: String,
    pub matched_capabilities: Vec<Capability>,
    pub reason: String,
}

#[derive(Debug, Default)]
pub struct Gateway {
    providers: BTreeMap<String, ProviderManifest>,
}

impl Gateway {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, manifest: ProviderManifest) -> Result<(), GatewayError> {
        if manifest.id.trim().is_empty() {
            return Err(GatewayError::InvalidProviderId);
        }
        if self.providers.contains_key(&manifest.id) {
            return Err(GatewayError::DuplicateProvider(manifest.id));
        }
        self.providers.insert(manifest.id.clone(), manifest);
        Ok(())
    }

    pub fn provider(&self, id: &str) -> Option<&ProviderManifest> {
        self.providers.get(id)
    }

    pub fn providers(&self) -> impl Iterator<Item = &ProviderManifest> {
        self.providers.values()
    }

    pub fn route(&self, request: &RouteRequest) -> Result<RouteDecision, GatewayError> {
        if let Some(preferred) = request.preferred_provider.as_deref() {
            let provider = self
                .providers
                .get(preferred)
                .ok_or_else(|| GatewayError::UnknownProvider(preferred.to_owned()))?;
            if supports_all(provider, &request.required) {
                return Ok(decision(provider, &request.required, "preferred provider matched"));
            }
        }

        self.providers
            .values()
            .filter(|provider| supports_all(provider, &request.required))
            .max_by_key(|provider| {
                let local_score = usize::from(request.prefer_local && provider.local) * 100;
                local_score + provider.capabilities.len()
            })
            .map(|provider| decision(provider, &request.required, "best capability match"))
            .ok_or_else(|| GatewayError::NoCompatibleProvider(request.required.clone()))
    }
}

fn supports_all(provider: &ProviderManifest, required: &[Capability]) -> bool {
    required.iter().all(|capability| provider.supports(*capability))
}

fn decision(
    provider: &ProviderManifest,
    required: &[Capability],
    reason: &str,
) -> RouteDecision {
    RouteDecision {
        provider_id: provider.id.clone(),
        matched_capabilities: required.to_vec(),
        reason: reason.to_owned(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayError {
    InvalidProviderId,
    DuplicateProvider(String),
    UnknownProvider(String),
    NoCompatibleProvider(Vec<Capability>),
}

impl std::fmt::Display for GatewayError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProviderId => write!(formatter, "provider id cannot be empty"),
            Self::DuplicateProvider(id) => write!(formatter, "provider already registered: {id}"),
            Self::UnknownProvider(id) => write!(formatter, "unknown provider: {id}"),
            Self::NoCompatibleProvider(required) => {
                write!(formatter, "no provider satisfies capabilities: {required:?}")
            }
        }
    }
}

impl std::error::Error for GatewayError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider(id: &str, local: bool, capabilities: Vec<Capability>) -> ProviderManifest {
        ProviderManifest {
            id: id.into(),
            display_name: id.into(),
            transport: Transport::OpenAiCompatible,
            local,
            capabilities,
            default_model: None,
        }
    }

    #[test]
    fn prefers_local_provider_when_requested() {
        let mut gateway = Gateway::new();
        gateway
            .register(provider(
                "cloud",
                false,
                vec![Capability::Chat, Capability::Streaming],
            ))
            .unwrap();
        gateway
            .register(provider(
                "local",
                true,
                vec![Capability::Chat, Capability::Streaming],
            ))
            .unwrap();

        let route = gateway
            .route(&RouteRequest {
                required: vec![Capability::Chat],
                prefer_local: true,
                preferred_provider: None,
            })
            .unwrap();
        assert_eq!(route.provider_id, "local");
    }

    #[test]
    fn rejects_provider_without_required_capability() {
        let mut gateway = Gateway::new();
        gateway
            .register(provider("text", true, vec![Capability::Chat]))
            .unwrap();
        assert!(matches!(
            gateway.route(&RouteRequest {
                required: vec![Capability::Vision],
                prefer_local: true,
                preferred_provider: None,
            }),
            Err(GatewayError::NoCompatibleProvider(_))
        ));
    }
}
