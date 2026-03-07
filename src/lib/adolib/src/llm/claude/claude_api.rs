use std::collections::HashMap;
use std::fs;

use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Error;
use crate::error::Result;
use crate::llm::claude::claude_config::ClaudeConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeErrorMessage {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeError {
    #[serde(rename = "type")]
    error_type: String,
    request_id: Option<String>,
    error: ClaudeErrorMessage,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ClaudeRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClaudeMessage {
    pub role: ClaudeRole,
    pub content: Value,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum ClaudeContentType {
    #[default]
    #[serde(rename = "text")]
    Text,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ClaudeContent {
    #[serde(rename = "type")]
    pub content_type: ClaudeContentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClaudeCacheCreation {
    ephemeral_5m_input_tokens: u64,
    ephemeral_1h_input_tokens: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClaudeUsage {
    pub input_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub cache_creation: ClaudeCacheCreation,
    pub output_tokens: u64,
    pub service_tier: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClaudeStopReason {
    #[serde(rename = "end_turn")]
    EndTurn,
    #[serde(rename = "max_tokens")]
    MaxToken,
    #[serde(rename = "stop_sequence")]
    StopSequence,
    #[serde(rename = "tool_use")]
    ToolUse,
    #[serde(rename = "pause_turn")]
    PauseTurn,
    #[serde(rename = "refusal")]
    Resusal,
}

impl core::fmt::Display for ClaudeStopReason {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClaudeResponseType {
    #[serde(rename = "message")]
    Message,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: ClaudeResponseType,
    pub role: ClaudeRole,
    pub model: String,
    pub content: Vec<ClaudeContent>,
    pub stop_reason: ClaudeStopReason,
    pub stop_sequence: Option<String>,
    pub usage: ClaudeUsage,
}

#[derive(Deserialize)]
pub struct ClaudeModel {
    pub id: String,
}

#[derive(Deserialize)]
pub struct ClaudeModelResponse {
    pub data: Vec<ClaudeModel>,
}

pub struct ClaudeApi {
    pub config: ClaudeConfig,
}

#[derive(Debug, Default, Serialize)]
pub struct ClaudeMessages {
    model: String,
    messages: Vec<ClaudeMessage>,
    max_tokens: u64,
    stream: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    system: Vec<ClaudeContent>,
}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////

impl ClaudeMessage {
    pub fn with_message<S>(role: ClaudeRole, message: S) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            role,
            content: Value::String(message.as_ref().to_string()),
        }
    }
}

impl ClaudeResponse {
    pub fn message(&self) -> Result<&str> {
        let text = self.content.first().ok_or(Error::Empty)?;

        match &text.text {
            Some(v) => Ok(v),
            None => Err(Error::Empty),
        }
    }
}

impl ClaudeMessages {
    pub fn new<M>(model: M, max_tokens: u64) -> Self
    where
        M: AsRef<str>,
    {
        Self {
            model: model.as_ref().to_string(),
            messages: vec![],
            max_tokens,
            stream: false,
            ..Default::default()
        }
    }

    pub fn add_system_prompt<S>(&mut self, text: S)
    where
        S: AsRef<str>,
    {
        let content = ClaudeContent {
            text: Some(text.as_ref().into()),
            content_type: ClaudeContentType::Text,
            ..Default::default()
        };
        self.system.push(content);
    }

    pub fn add_message<C>(&mut self, role: ClaudeRole, message: C)
    where
        C: AsRef<str>,
    {
        let message = ClaudeMessage::with_message(role, message);
        self.messages.push(message);
    }

    pub fn reset(&mut self) {
        self.messages = vec![]
    }
}

impl ClaudeApi {
    pub fn new(config: &ClaudeConfig) -> Result<Self> {
        Ok(Self { config: config.clone() })
    }

    pub fn models(&self) -> Result<Vec<ClaudeModel>> {
        let url = format!("{}/v1/models", self.config.url);

        let mut res = ureq::get(&url)
            .header("x-api-key", &self.config.key)
            .header("anthropic-version", &self.config.anthropic_version)
            .call()?;

        let log_msg = format!(
            "get {} -> code={} reason={}",
            url,
            res.status().as_u16(),
            res.status().as_str()
        );

        match res.status().is_success() {
            true => info!("{log_msg}"),
            false => error!("{log_msg}"),
        }

        let resp_json = &res.body_mut().read_to_string()?;

        let resp: ClaudeModelResponse = serde_json::from_str(&resp_json)?;

        Ok(resp.data)
    }

    pub fn chat(&self, chat: &ClaudeMessages) -> Result<ClaudeResponse> {
        let req_json = serde_json::to_string_pretty(&chat)?;

        let url = format!("{}/v1/messages", self.config.url);

        let mut res = ureq::post(&url)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.config.key)
            .header("anthropic-version", &self.config.anthropic_version)
            .send(req_json)?;

        let log_msg = format!(
            "post {} -> code={} reason={}",
            url,
            res.status().as_u16(),
            res.status().as_str()
        );

        match res.status().is_success() {
            true => info!("{log_msg}"),
            false => error!("{log_msg}"),
        }

        let success = res.status().is_success();

        let resp_json = &res.body_mut().read_to_string()?;

        if !success {
            error!("{resp_json}")
        }

        // convert a our struct
        match serde_json::from_str::<ClaudeResponse>(resp_json) {
            Ok(v) => Ok(v),
            Err(_) => {
                //
                // Try to print it nicely, best effort
                //
                fs::write("/tmp/claude_serde_error.json", resp_json.as_bytes())?;
                let claude_error: ClaudeError = serde_json::from_str(resp_json)?;
                let pretty_error = serde_json::to_string_pretty(&claude_error)?;
                let pretty_error = format!("# Claude Error\n\n```json\n{pretty_error}\n```");
                Err(Error::LlmError { message: pretty_error })
            }
        }
    }

    pub fn message<S>(&self, content: S) -> Result<ClaudeResponse>
    where
        S: AsRef<str>,
    {
        let mut chat = ClaudeMessages::new(&self.config.model, 4096);

        chat.add_message(ClaudeRole::User, content);

        self.chat(&chat)
    }
}

///////////////////////////////////////////////////////////////////////////////
// TESTS
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use log::info;

    use crate::llm::claude::claude_api::ClaudeResponse;

    #[test]
    fn test_response() {
        let test_file = Path::new("/tmp").join("claude_response.json");

        let resp = fs::read_to_string(test_file).unwrap();

        let resp: ClaudeResponse = serde_json::from_str(&resp).unwrap();

        info!("{resp:?}");
    }
}
