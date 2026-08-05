// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::{collections::BTreeMap, io::Read, time::Duration};

use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;

use crate::model::ModelError;

#[derive(Debug, Clone)]
pub struct HttpTransport {
    client: Client,
    max_response_bytes: usize,
}

impl HttpTransport {
    pub fn new(timeout_ms: u64, max_response_bytes: usize) -> Result<Self, ModelError> {
        let client = Client::builder()
            .connect_timeout(Duration::from_millis(timeout_ms.min(30_000)))
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .map_err(|error| ModelError::Unavailable(error.to_string()))?;
        Ok(Self {
            client,
            max_response_bytes: max_response_bytes.max(1024),
        })
    }

    pub fn get_json(
        &self,
        url: &str,
        headers: &BTreeMap<String, String>,
        bearer: Option<&str>,
    ) -> Result<Value, ModelError> {
        let response = self
            .client
            .get(url)
            .headers(build_headers(headers, bearer)?)
            .send()
            .map_err(map_reqwest_error)?;
        decode_json(response, self.max_response_bytes)
    }

    pub fn post_json(
        &self,
        url: &str,
        headers: &BTreeMap<String, String>,
        bearer: Option<&str>,
        body: &Value,
    ) -> Result<Value, ModelError> {
        let response = self
            .client
            .post(url)
            .headers(build_headers(headers, bearer)?)
            .header(CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .map_err(map_reqwest_error)?;
        decode_json(response, self.max_response_bytes)
    }

    pub fn post_stream(
        &self,
        url: &str,
        headers: &BTreeMap<String, String>,
        bearer: Option<&str>,
        body: &Value,
        mut on_chunk: impl FnMut(&str) -> Result<(), ModelError>,
    ) -> Result<(), ModelError> {
        let mut response = self
            .client
            .post(url)
            .headers(build_headers(headers, bearer)?)
            .header(CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .map_err(map_reqwest_error)?;
        ensure_success(&response)?;

        let mut total = 0usize;
        let mut buffer = [0u8; 8192];
        loop {
            let read = response
                .read(&mut buffer)
                .map_err(|error| ModelError::Unavailable(error.to_string()))?;
            if read == 0 {
                break;
            }
            total = total.saturating_add(read);
            if total > self.max_response_bytes {
                return Err(ModelError::Protocol("stream exceeded response limit".into()));
            }
            let chunk = std::str::from_utf8(&buffer[..read])
                .map_err(|_| ModelError::Protocol("provider stream was not UTF-8".into()))?;
            on_chunk(chunk)?;
        }
        Ok(())
    }
}

fn build_headers(
    custom: &BTreeMap<String, String>,
    bearer: Option<&str>,
) -> Result<HeaderMap, ModelError> {
    let mut headers = HeaderMap::new();
    for (name, value) in custom {
        let name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| ModelError::InvalidRequest(format!("invalid header name: {name}")))?;
        let value = HeaderValue::from_str(value)
            .map_err(|_| ModelError::InvalidRequest("invalid header value".into()))?;
        headers.insert(name, value);
    }
    if let Some(token) = bearer {
        let value = HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|_| ModelError::Authentication)?;
        headers.insert(AUTHORIZATION, value);
    }
    Ok(headers)
}

fn decode_json(response: Response, limit: usize) -> Result<Value, ModelError> {
    ensure_success(&response)?;
    let bytes = response.bytes().map_err(map_reqwest_error)?;
    if bytes.len() > limit {
        return Err(ModelError::Protocol("response exceeded size limit".into()));
    }
    serde_json::from_slice(&bytes)
        .map_err(|error| ModelError::Protocol(format!("invalid provider JSON: {error}")))
}

fn ensure_success(response: &Response) -> Result<(), ModelError> {
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }
    match status.as_u16() {
        401 | 403 => Err(ModelError::Authentication),
        429 => Err(ModelError::RateLimited),
        408 | 504 => Err(ModelError::Timeout),
        code => Err(ModelError::Unavailable(format!("HTTP {code}"))),
    }
}

fn map_reqwest_error(error: reqwest::Error) -> ModelError {
    if error.is_timeout() {
        ModelError::Timeout
    } else if error.is_connect() {
        ModelError::Unavailable("connection failed".into())
    } else {
        ModelError::Unavailable(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_custom_header_name() {
        let mut headers = BTreeMap::new();
        headers.insert("bad header".into(), "x".into());
        assert!(build_headers(&headers, None).is_err());
    }
}
