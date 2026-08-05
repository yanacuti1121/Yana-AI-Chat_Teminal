// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use serde_json::{json, Value};

use crate::{
    adapters::{AdapterConfig, ModelCapability, ModelDescriptor, ProviderAdapter},
    http_transport::HttpTransport,
    model::{FinishReason, ModelError, ModelRequest, ModelResponse, Role, StreamEvent, Usage},
    streaming::{StreamDecoder, StreamFormat},
};

pub struct OllamaAdapter {
    config: AdapterConfig,
    transport: HttpTransport,
}

impl OllamaAdapter {
    pub fn new(config: AdapterConfig) -> Result<Self, ModelError> {
        config.validate()?;
        let transport = HttpTransport::new(config.timeout_ms, 32 * 1024 * 1024)?;
        Ok(Self { config, transport })
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}/{}", self.config.endpoint.trim_end_matches('/'), path)
    }
}

impl ProviderAdapter for OllamaAdapter {
    fn id(&self) -> &str { &self.config.id }
    fn config(&self) -> &AdapterConfig { &self.config }

    fn discover_models(&self) -> Result<Vec<ModelDescriptor>, ModelError> {
        let value = self.transport.get_json(&self.endpoint("api/tags"), &self.config.headers, None)?;
        let models = value.get("models").and_then(Value::as_array)
            .ok_or_else(|| ModelError::Protocol("Ollama tags response missing models".into()))?;
        Ok(models.iter().filter_map(|item| {
            let id = item.get("name").or_else(|| item.get("model"))?.as_str()?.to_owned();
            Some(ModelDescriptor {
                display_name: id.clone(),
                id,
                context_window: 0,
                capabilities: vec![ModelCapability::Chat, ModelCapability::Streaming],
                local: true,
            })
        }).collect())
    }

    fn complete(&self, request: &ModelRequest) -> Result<ModelResponse, ModelError> {
        request.validate()?;
        if !request.tools.is_empty() {
            return Err(ModelError::UnsupportedCapability("tool calling for Ollama adapter".into()));
        }
        let value = self.transport.post_json(
            &self.endpoint("api/chat"),
            &self.config.headers,
            None,
            &request_body(request, false),
        )?;
        decode_response(&value)
    }

    fn stream(
        &self,
        request: &ModelRequest,
        emit: &mut dyn FnMut(StreamEvent) -> Result<(), ModelError>,
    ) -> Result<(), ModelError> {
        request.validate()?;
        if !request.tools.is_empty() {
            return Err(ModelError::UnsupportedCapability("tool calling for Ollama adapter".into()));
        }
        let mut decoder = StreamDecoder::default();
        emit(StreamEvent::Started)?;
        self.transport.post_stream(
            &self.endpoint("api/chat"),
            &self.config.headers,
            None,
            &request_body(request, true),
            |chunk| {
                for event in decoder.push(StreamFormat::NewlineDelimitedJson, chunk)? {
                    emit(event)?;
                }
                Ok(())
            },
        )?;
        if let Some(event) = decoder.finish()? { emit(event)?; }
        Ok(())
    }
}

fn request_body(request: &ModelRequest, stream: bool) -> Value {
    let messages = request.messages.iter().map(|message| json!({
        "role": match message.role { Role::System => "system", Role::User => "user", Role::Assistant => "assistant", Role::Tool => "tool" },
        "content": message.content,
    })).collect::<Vec<_>>();
    let mut options = serde_json::Map::new();
    if let Some(value) = request.temperature { options.insert("temperature".into(), json!(value)); }
    if let Some(value) = request.max_output_tokens { options.insert("num_predict".into(), json!(value)); }
    json!({ "model": request.model, "messages": messages, "stream": stream, "options": options })
}

fn decode_response(value: &Value) -> Result<ModelResponse, ModelError> {
    let text = value.pointer("/message/content").and_then(Value::as_str)
        .ok_or_else(|| ModelError::Protocol("Ollama response missing message content".into()))?
        .to_owned();
    Ok(ModelResponse {
        text,
        tool_calls: Vec::new(),
        finish_reason: if value.get("done").and_then(Value::as_bool).unwrap_or(false) { FinishReason::Stop } else { FinishReason::Unknown },
        usage: Usage {
            input_tokens: value.get("prompt_eval_count").and_then(Value::as_u64).unwrap_or(0),
            output_tokens: value.get("eval_count").and_then(Value::as_u64).unwrap_or(0),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_ollama_chat_response() {
        let response = decode_response(&json!({"message":{"content":"ok"},"done":true,"eval_count":3})).unwrap();
        assert_eq!(response.text, "ok");
        assert_eq!(response.usage.output_tokens, 3);
    }
}
