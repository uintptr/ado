use std::collections::HashMap;

use derive_more::Display;
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::error::Result;
use crate::llm::claude::claude_config::ClaudeConfig;
use crate::llm::claude::claude_config::ClaudeMcpServer;
use crate::llm::claude::claude_tool::ClaudeTool;
use crate::tools::loader::Tools;

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
    pub content: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum ClaudeContentType {
    #[default]
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "tool_use")]
    ToolUse,
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
    pub input: Option<HashMap<String, String>>,
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

#[derive(Debug, Display, Serialize, Deserialize)]
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

pub struct ClaudeApi {
    client: Client,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<ClaudeTool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    mcp_servers: Vec<ClaudeMcpServer>,
}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////

impl ClaudeMessage {
    pub fn new<C>(role: ClaudeRole, content: C) -> Self
    where
        C: AsRef<str>,
    {
        Self {
            role,
            content: content.as_ref().to_string(),
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

    pub fn with_tools(&mut self, tools: Tools) {
        self.tools.clear();

        for t in tools.list {
            let claude_tool: ClaudeTool = match t.try_into() {
                Ok(v) => v,
                Err(e) => {
                    error!("{e}");
                    continue;
                }
            };
            self.tools.push(claude_tool);
        }
    }

    pub fn without_tools(&mut self) {
        self.tools.clear();
    }

    pub fn add_content<C>(&mut self, role: ClaudeRole, content: C)
    where
        C: AsRef<str>,
    {
        let message = ClaudeMessage::new(role, content);
        self.messages.push(message);
    }

    pub fn reset(&mut self) {
        self.messages = vec![]
    }
}

impl ClaudeApi {
    pub fn new(config: &ClaudeConfig) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            config: config.clone(),
        })
    }

    pub async fn chat(&self, chat: &ClaudeMessages) -> Result<ClaudeResponse> {
        let req_json = serde_json::to_string_pretty(&chat)?;

        let url = format!("{}/v1/messages", self.config.url);

        let req_builds = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.config.key)
            .header("anthropic-version", &self.config.anthropic_version)
            .body(req_json);

        let req_builds = match self.config.mcp_servers {
            Some(_) => req_builds.header("anthropic-beta", "mcp-client-2025-04-04"),
            None => req_builds,
        };

        let res = req_builds.send().await?;

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

        let resp_json = &res.text().await?;

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
                let claude_error: ClaudeError = serde_json::from_str(resp_json)?;
                let pretty_error = serde_json::to_string_pretty(&claude_error)?;
                let pretty_error = format!("# Claude Error\n\n```json\n{pretty_error}\n```");
                Err(Error::LlmError { message: pretty_error })
            }
        }
    }

    pub async fn message<S>(&self, content: S) -> Result<ClaudeResponse>
    where
        S: AsRef<str>,
    {
        let mut chat = ClaudeMessages::new(&self.config.model, 4096);

        chat.add_content(ClaudeRole::User, content.as_ref());

        self.chat(&chat).await
    }
}

///////////////////////////////////////////////////////////////////////////////
// TESTS
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use log::info;

    use crate::{llm::claude::claude_api::ClaudeResponse, logging::logger::setup_logger};

    #[test]
    fn test_response() {
        setup_logger(true).unwrap();
        let test_file = Path::new("/tmp").join("claude_response.json");

        let resp = fs::read_to_string(test_file).unwrap();

        let resp: ClaudeResponse = serde_json::from_str(&resp).unwrap();

        info!("{resp:?}");
    }
}
