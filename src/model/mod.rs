// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub tools: Vec<ToolSpec>,
    pub temperature: Option<f32>,
    pub max_output_tokens: Option<u32>,
    pub stream: bool,
}

impl ModelRequest {
    pub fn validate(&self) -> Result<(), ModelError> {
        if self.model.trim().is_empty() {
            return Err(ModelError::InvalidRequest("model cannot be empty".into()));
        }
        if self.messages.is_empty() {
            return Err(ModelError::InvalidRequest("messages cannot be empty".into()));
        }
        if let Some(temperature) = self.temperature {
            if !(0.0..=2.0).contains(&temperature) {
                return Err(ModelError::InvalidRequest(
                    "temperature must be between 0 and 2".into(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelResponse {
    pub text: String,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: FinishReason,
    pub usage: Usage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCall,
    Cancelled,
    Error,
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamEvent {
    Started,
    TextDelta(String),
    ToolCallDelta { id: String, name: String, arguments: String },
    Usage(Usage),
    Finished(FinishReason),
    Error(ModelError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    InvalidRequest(String),
    UnsupportedCapability(String),
    Authentication,
    RateLimited,
    Timeout,
    Cancelled,
    Unavailable(String),
    Protocol(String),
}

impl ModelError {
    pub fn retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited | Self::Timeout | Self::Unavailable(_)
        )
    }
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequest(message) => write!(formatter, "invalid model request: {message}"),
            Self::UnsupportedCapability(capability) => {
                write!(formatter, "provider does not support {capability}")
            }
            Self::Authentication => write!(formatter, "provider authentication failed"),
            Self::RateLimited => write!(formatter, "provider rate limit reached"),
            Self::Timeout => write!(formatter, "provider request timed out"),
            Self::Cancelled => write!(formatter, "provider request cancelled"),
            Self::Unavailable(message) => write!(formatter, "provider unavailable: {message}"),
            Self::Protocol(message) => write!(formatter, "provider protocol error: {message}"),
        }
    }
}

impl std::error::Error for ModelError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_model() {
        let request = ModelRequest {
            model: "".into(),
            messages: vec![Message {
                role: Role::User,
                content: "hello".into(),
            }],
            tools: Vec::new(),
            temperature: None,
            max_output_tokens: None,
            stream: true,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn retryability_is_explicit() {
        assert!(ModelError::Timeout.retryable());
        assert!(!ModelError::Authentication.retryable());
    }
}
