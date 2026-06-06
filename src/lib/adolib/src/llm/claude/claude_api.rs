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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClaudeCacheControl {
    #[serde(rename = "type")]
    pub cache_type: String,
}

impl ClaudeCacheControl {
    #[must_use]
    pub fn ephemeral() -> Self {
        Self {
            cache_type: "ephemeral".to_string(),
        }
    }
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
    /// Marks a prompt-caching breakpoint; everything up to and including this
    /// block is cached. Only ever set on the last system block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<ClaudeCacheControl>,
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
        let s = match self {
            Self::EndTurn => "EndTurn",
            Self::MaxToken => "MaxToken",
            Self::StopSequence => "StopSequence",
            Self::ToolUse => "ToolUse",
            Self::PauseTurn => "PauseTurn",
            Self::Resusal => "Resusal",
        };
        write!(f, "{s}")
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
    #[serde(skip_serializing_if = "Option::is_none")]
    output_config: Option<Value>,
}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////

impl ClaudeMessage {
    pub fn with_message<S>(role: ClaudeRole, message: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            role,
            content: Value::String(message.into()),
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
        C: Into<String>,
    {
        let message = ClaudeMessage::with_message(role, message);
        self.messages.push(message);
    }

    pub fn reset(&mut self) {
        self.messages = vec![];
    }

    /// Constrain responses to the given JSON schema via Anthropic structured
    /// outputs, so the model can only emit schema-valid JSON.
    pub fn set_output_schema(&mut self, schema: &Value) {
        self.output_config = Some(serde_json::json!({
            "format": {
                "type": "json_schema",
                "schema": schema,
            }
        }));
    }

    /// Place prompt-caching breakpoints just before sending a request.
    ///
    /// Caching is a prefix match (render order: tools → system → messages), so
    /// we set two `cache_control` markers:
    ///   1. the last `system` block — caches the stable prompt prefix, and
    ///   2. the last message — caches the growing conversation prefix (which
    ///      includes the system prompts) incrementally across turns.
    ///
    /// Both are idempotent: any previous message breakpoint is stripped first
    /// so we never exceed the 4-breakpoint limit. Note: a prefix shorter than
    /// the model's minimum (4096 tokens on Opus) silently won't cache.
    pub fn set_cache_breakpoints(&mut self) {
        // System prefix: keep exactly one breakpoint, on the last block.
        for block in &mut self.system {
            block.cache_control = None;
        }
        if let Some(last) = self.system.last_mut() {
            last.cache_control = Some(ClaudeCacheControl::ephemeral());
        }

        // Conversation prefix: move the breakpoint to the current last message.
        // Revert any prior block-form content back to a plain string first.
        for msg in &mut self.messages {
            if msg.content.is_array() {
                let text = msg
                    .content
                    .get(0)
                    .and_then(|b| b.get("text"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                msg.content = Value::String(text);
            }
        }
        if let Some(last) = self.messages.last_mut() {
            let text = last.content.as_str().unwrap_or_default().to_string();
            last.content = serde_json::json!([{
                "type": "text",
                "text": text,
                "cache_control": { "type": "ephemeral" }
            }]);
        }
    }
}

impl ClaudeApi {
    #[must_use]
    pub fn new(config: &ClaudeConfig) -> Self {
        Self { config: config.clone() }
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

        if res.status().is_success() {
            info!("{log_msg}");
        } else {
            error!("{log_msg}");
        }

        let resp_json = &res.body_mut().read_to_string()?;

        let resp: ClaudeModelResponse = serde_json::from_str(resp_json)?;

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

        if res.status().is_success() {
            info!("{log_msg}");
        } else {
            error!("{log_msg}");
        }

        let success = res.status().is_success();

        let resp_json = &res.body_mut().read_to_string()?;

        if !success {
            error!("{resp_json}");
        }

        // convert a our struct
        if let Ok(v) = serde_json::from_str::<ClaudeResponse>(resp_json) {
            Ok(v)
        } else {
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

    pub fn message<S>(&self, content: S) -> Result<ClaudeResponse>
    where
        S: Into<String>,
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
    use crate::llm::claude::claude_api::{ClaudeMessages, ClaudeResponse, ClaudeRole};

    #[test]
    fn test_response() {
        let json = r#"{
            "id": "msg_01XFDUDYJgAACzvnptvVoYEL",
            "type": "message",
            "role": "assistant",
            "model": "claude-sonnet-4-6",
            "content": [{ "type": "text", "text": "Hello!" }],
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {
                "input_tokens": 10,
                "cache_creation_input_tokens": 0,
                "cache_read_input_tokens": 0,
                "cache_creation": {
                    "ephemeral_5m_input_tokens": 0,
                    "ephemeral_1h_input_tokens": 0
                },
                "output_tokens": 5,
                "service_tier": "standard"
            }
        }"#;

        let resp: ClaudeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.content.first().and_then(|c| c.text.as_deref()), Some("Hello!"));
    }

    #[test]
    fn test_cache_breakpoints() {
        let mut chat = ClaudeMessages::new("claude-opus-4-7", 1024);
        chat.add_system_prompt("system A");
        chat.add_system_prompt("system B");
        chat.add_message(ClaudeRole::User, "hello");
        chat.add_message(ClaudeRole::Assistant, "hi");
        chat.add_message(ClaudeRole::User, "again");

        chat.set_cache_breakpoints();

        // Only the LAST system block carries a breakpoint.
        assert!(chat.system[0].cache_control.is_none());
        assert!(chat.system[1].cache_control.is_some());

        // Only the LAST message is converted to a cached text block.
        assert!(chat.messages[0].content.is_string());
        assert!(chat.messages[1].content.is_string());
        let last = &chat.messages[2].content;
        assert_eq!(last[0]["type"], "text");
        assert_eq!(last[0]["text"], "again");
        assert_eq!(last[0]["cache_control"]["type"], "ephemeral");

        // Idempotent: a second pass moves the message breakpoint to the new
        // last message and reverts the previous one — never accumulating.
        chat.add_message(ClaudeRole::Assistant, "ok");
        chat.set_cache_breakpoints();
        assert!(chat.messages[2].content.is_string()); // reverted
        assert_eq!(chat.messages[3].content[0]["cache_control"]["type"], "ephemeral");

        // At most one system + one message breakpoint (well under the limit of 4).
        let system_bps = chat.system.iter().filter(|b| b.cache_control.is_some()).count();
        let msg_bps = chat.messages.iter().filter(|m| m.content.is_array()).count();
        assert_eq!(system_bps, 1);
        assert_eq!(msg_bps, 1);
    }
}
