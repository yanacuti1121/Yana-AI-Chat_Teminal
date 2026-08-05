// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::env;

use serde_json::{json, Value};

use crate::{
    adapters::{AdapterConfig, ModelCapability, ModelDescriptor, ProviderAdapter},
    http_transport::HttpTransport,
    model::{FinishReason, ModelError, ModelRequest, ModelResponse, Role, StreamEvent, ToolCall, Usage},
    streaming::{StreamDecoder, StreamFormat},
};

pub struct OpenAiCompatibleAdapter {
    config: AdapterConfig,
    transport: HttpTransport,
}

impl OpenAiCompatibleAdapter {
    pub fn new(config: AdapterConfig) -> Result<Self, ModelError> {
        config.validate()?;
        let transport = HttpTransport::new(config.timeout_ms, 16 * 1024 * 1024)?;
        Ok(Self { config, transport })
    }

    fn bearer(&self) -> Result<Option<String>, ModelError> {
        match &self.config.api_key_env {
            Some(name) => env::var(name).map(Some).map_err(|_| ModelError::Authentication),
            None => Ok(None),
        }
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}/{}", self.config.endpoint.trim_end_matches('/'), path)
    }
}

impl ProviderAdapter for OpenAiCompatibleAdapter {
    fn id(&self) -> &str { &self.config.id }
    fn config(&self) -> &AdapterConfig { &self.config }

    fn discover_models(&self) -> Result<Vec<ModelDescriptor>, ModelError> {
        let bearer = self.bearer()?;
        let value = self.transport.get_json(
            &self.endpoint("v1/models"),
            &self.config.headers,
            bearer.as_deref(),
        )?;
        let data = value.get("data").and_then(Value::as_array)
            .ok_or_else(|| ModelError::Protocol("models response missing data array".into()))?;
        Ok(data.iter().filter_map(|item| item.get("id").and_then(Value::as_str)).map(|id| ModelDescriptor {
            id: id.to_owned(),
            display_name: id.to_owned(),
            context_window: 0,
            capabilities: vec![ModelCapability::Chat, ModelCapability::Streaming, ModelCapability::ToolCalling],
            local: false,
        }).collect())
    }

    fn complete(&self, request: &ModelRequest) -> Result<ModelResponse, ModelError> {
        request.validate()?;
        let bearer = self.bearer()?;
        let body = request_body(request, false);
        let value = self.transport.post_json(
            &self.endpoint("v1/chat/completions"),
            &self.config.headers,
            bearer.as_deref(),
            &body,
        )?;
        decode_response(&value)
    }

    fn stream(
        &self,
        request: &ModelRequest,
        emit: &mut dyn FnMut(StreamEvent) -> Result<(), ModelError>,
    ) -> Result<(), ModelError> {
        request.validate()?;
        let bearer = self.bearer()?;
        let body = request_body(request, true);
        let mut decoder = StreamDecoder::default();
        emit(StreamEvent::Started)?;
        self.transport.post_stream(
            &self.endpoint("v1/chat/completions"),
            &self.config.headers,
            bearer.as_deref(),
            &body,
            |chunk| {
                for event in decoder.push(StreamFormat::ServerSentEvents, chunk)? {
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
    let tools = request.tools.iter().map(|tool| json!({
        "type": "function",
        "function": { "name": tool.name, "description": tool.description, "parameters": tool.input_schema }
    })).collect::<Vec<_>>();
    let mut body = json!({ "model": request.model, "messages": messages, "stream": stream });
    if !tools.is_empty() { body["tools"] = Value::Array(tools); }
    if let Some(value) = request.temperature { body["temperature"] = json!(value); }
    if let Some(value) = request.max_output_tokens { body["max_tokens"] = json!(value); }
    body
}

fn decode_response(value: &Value) -> Result<ModelResponse, ModelError> {
    let choice = value.pointer("/choices/0")
        .ok_or_else(|| ModelError::Protocol("completion response missing first choice".into()))?;
    let text = choice.pointer("/message/content").and_then(Value::as_str).unwrap_or_default().to_owned();
    let tool_calls = choice.pointer("/message/tool_calls").and_then(Value::as_array).map(|calls| {
        calls.iter().filter_map(|call| {
            let id = call.get("id")?.as_str()?.to_owned();
            let name = call.pointer("/function/name")?.as_str()?.to_owned();
            let raw = call.pointer("/function/arguments")?.as_str()?;
            let arguments = serde_json::from_str(raw).unwrap_or_else(|_| json!({"raw": raw}));
            Some(ToolCall { id, name, arguments })
        }).collect()
    }).unwrap_or_default();
    let reason = match choice.get("finish_reason").and_then(Value::as_str) {
        Some("stop") => FinishReason::Stop,
        Some("length") => FinishReason::Length,
        Some("tool_calls") => FinishReason::ToolCall,
        Some(_) => FinishReason::Unknown,
        None => FinishReason::Unknown,
    };
    let usage = Usage {
        input_tokens: value.pointer("/usage/prompt_tokens").and_then(Value::as_u64).unwrap_or(0),
        output_tokens: value.pointer("/usage/completion_tokens").and_then(Value::as_u64).unwrap_or(0),
    };
    Ok(ModelResponse { text, tool_calls, finish_reason: reason, usage })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_basic_completion() {
        let value = json!({"choices":[{"message":{"content":"hello"},"finish_reason":"stop"}],"usage":{"prompt_tokens":2,"completion_tokens":1}});
        let response = decode_response(&value).unwrap();
        assert_eq!(response.text, "hello");
        assert_eq!(response.usage.input_tokens, 2);
    }
}
