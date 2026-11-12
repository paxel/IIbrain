//! A Model: a language model the person connected, reached through an
//! OpenAI-compatible HTTP endpoint (ADR 0003). The app brings none of
//! its own.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::error::{Error, Result};

/// How long the app waits for a Model's answer.
const TIMEOUT: Duration = Duration::from_secs(120);
/// How long it waits when it only tests the connection.
const TEST_TIMEOUT: Duration = Duration::from_secs(15);

/// A connected Model. It belongs to one Device; its address and key
/// never enter the Brain's folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    /// What the person calls it; no two Models of a Device share it.
    pub name: String,
    /// The endpoint's base address, such as `http://localhost:11434/v1`.
    pub address: String,
    /// The key the endpoint asks for, if any.
    #[serde(default)]
    pub key: Option<String>,
    /// The model's name at the endpoint.
    pub model: String,
    /// Whether the person declared it a Local Model: one running under
    /// their own control. The app takes their word for it.
    pub local: bool,
}

/// One turn of what is sent to a Model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// `system` or `user`.
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn system(content: &str) -> Self {
        Self {
            role: "system".to_string(),
            content: content.to_string(),
        }
    }

    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: content.to_string(),
        }
    }
}

impl Model {
    /// Sends `messages` and gives what the Model wrote back.
    pub fn ask(&self, messages: &[Message]) -> Result<String> {
        self.send(messages, TIMEOUT, None)
    }

    /// Tries the connection with one short message, so that a wrong
    /// address, key or model name shows when the Model is added.
    pub fn test(&self) -> Result<()> {
        self.send(&[Message::user("Hi")], TEST_TIMEOUT, Some(1))
            .map(|_| ())
    }

    fn send(&self, messages: &[Message], timeout: Duration, limit: Option<u32>) -> Result<String> {
        let address = self.address.trim().trim_end_matches('/');
        if !(address.starts_with("http://") || address.starts_with("https://")) {
            return Err(Error::Model(format!("{address} is no web address")));
        }
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(timeout))
            .http_status_as_error(false)
            .user_agent("IIbrain")
            .build()
            .into();
        let mut body = json!({ "model": self.model, "messages": messages, "stream": false });
        if let (Some(limit), Some(fields)) = (limit, body.as_object_mut()) {
            fields.insert("max_tokens".to_string(), json!(limit));
        }
        let mut request = agent.post(format!("{address}/chat/completions"));
        if let Some(key) = self.key.as_deref().filter(|k| !k.trim().is_empty()) {
            request = request.header("Authorization", format!("Bearer {}", key.trim()));
        }
        let mut response = request
            .send_json(&body)
            .map_err(|e| Error::Model(format!("not reached: {e}")))?;
        let status = response.status().as_u16();
        let text = response
            .body_mut()
            .with_config()
            .lossy_utf8(true)
            .read_to_string()
            .map_err(|e| Error::Model(e.to_string()))?;
        let answer: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
        if !(200..300).contains(&status) {
            // The endpoint's own words, when it gave any.
            let said = answer
                .pointer("/error/message")
                .or_else(|| answer.pointer("/error"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            let what = match status {
                401 | 403 => "the key was refused",
                404 => "the address or the model's name is not known there",
                429 => "the endpoint asks to wait",
                _ => "the endpoint refused",
            };
            return Err(Error::Model(
                format!("{what} ({status}) {said}").trim().to_string(),
            ));
        }
        answer
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .map(|content| content.trim().to_string())
            .ok_or_else(|| Error::Model("the answer was not in the expected form".to_string()))
    }
}
