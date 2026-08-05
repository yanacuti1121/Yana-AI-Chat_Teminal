// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use crate::model::{FinishReason, ModelError, StreamEvent, Usage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamFormat {
    ServerSentEvents,
    NewlineDelimitedJson,
}

#[derive(Debug, Default)]
pub struct StreamDecoder {
    buffer: String,
}

impl StreamDecoder {
    pub fn push(
        &mut self,
        format: StreamFormat,
        chunk: &str,
    ) -> Result<Vec<StreamEvent>, ModelError> {
        self.buffer.push_str(chunk);
        let mut events = Vec::new();

        loop {
            let boundary = match format {
                StreamFormat::ServerSentEvents => self.buffer.find("\n\n"),
                StreamFormat::NewlineDelimitedJson => self.buffer.find('\n'),
            };
            let Some(boundary) = boundary else { break };

            let frame = match format {
                StreamFormat::ServerSentEvents => self.buffer.drain(..boundary + 2).collect::<String>(),
                StreamFormat::NewlineDelimitedJson => {
                    self.buffer.drain(..boundary + 1).collect::<String>()
                }
            };
            if let Some(event) = decode_frame(format, frame.trim())? {
                events.push(event);
            }
        }

        Ok(events)
    }

    pub fn finish(self) -> Result<Option<StreamEvent>, ModelError> {
        if self.buffer.trim().is_empty() {
            return Ok(None);
        }
        Err(ModelError::Protocol(
            "stream ended with an incomplete frame".into(),
        ))
    }
}

fn decode_frame(format: StreamFormat, frame: &str) -> Result<Option<StreamEvent>, ModelError> {
    if frame.is_empty() || frame.starts_with(':') {
        return Ok(None);
    }

    let payload = match format {
        StreamFormat::ServerSentEvents => frame
            .lines()
            .filter_map(|line| line.strip_prefix("data:"))
            .map(str::trim_start)
            .collect::<Vec<_>>()
            .join("\n"),
        StreamFormat::NewlineDelimitedJson => frame.to_owned(),
    };

    if payload == "[DONE]" {
        return Ok(Some(StreamEvent::Finished(FinishReason::Stop)));
    }

    let value: serde_json::Value = serde_json::from_str(&payload)
        .map_err(|error| ModelError::Protocol(format!("invalid stream JSON: {error}")))?;

    if let Some(error) = value.get("error") {
        return Ok(Some(StreamEvent::Error(ModelError::Unavailable(
            error.to_string(),
        ))));
    }
    if let Some(text) = value
        .get("text")
        .or_else(|| value.pointer("/message/content"))
        .or_else(|| value.pointer("/choices/0/delta/content"))
        .and_then(serde_json::Value::as_str)
    {
        if !text.is_empty() {
            return Ok(Some(StreamEvent::TextDelta(text.to_owned())));
        }
    }
    if let Some(done) = value.get("done").and_then(serde_json::Value::as_bool) {
        if done {
            let usage = Usage {
                input_tokens: value
                    .get("prompt_eval_count")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
                output_tokens: value
                    .get("eval_count")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
            };
            if usage != Usage::default() {
                return Ok(Some(StreamEvent::Usage(usage)));
            }
            return Ok(Some(StreamEvent::Finished(FinishReason::Stop)));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_openai_sse_text() {
        let mut decoder = StreamDecoder::default();
        let events = decoder
            .push(
                StreamFormat::ServerSentEvents,
                "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\n",
            )
            .unwrap();
        assert_eq!(events, vec![StreamEvent::TextDelta("hello".into())]);
    }

    #[test]
    fn buffers_split_ndjson_frames() {
        let mut decoder = StreamDecoder::default();
        assert!(decoder
            .push(StreamFormat::NewlineDelimitedJson, "{\"text\":\"he")
            .unwrap()
            .is_empty());
        let events = decoder
            .push(StreamFormat::NewlineDelimitedJson, "llo\"}\n")
            .unwrap();
        assert_eq!(events, vec![StreamEvent::TextDelta("hello".into())]);
    }
}
