use std::collections::HashMap;

use derive_more::Display;
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::llm::claude::claude_tool::ClaudeTool;
use crate::tools::loader::Tools;
use crate::{config::loader::ClaudeAiConfig, error::Result};

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
pub struct ClaudeUsage {
    input_tokens: u64,
    output_tokens: u64,
}

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
#[derive(Debug, Display, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct ClaudeResponse {
    pub content: Vec<ClaudeContent>,
    //pub id: String,
    //pub model: String,
    pub role: ClaudeRole,
    pub stop_reason: ClaudeStopReason,
    //pub stop_sequence: Option<String>,
    //#[serde(rename = "type")]
    //pub response_type: String,
    //pub usage: ClaudeUsage,
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

pub struct ClaudeApi {
    client: Client,
    pub model: String,
    pub url: String,
    pub anthropic_version: String,
    pub key: String,
}

#[derive(Debug, Serialize)]
pub struct ClaudeChat {
    model: String,
    messages: Vec<ClaudeMessage>,
    max_tokens: u64,
    stream: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    system: Vec<ClaudeContent>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<ClaudeTool>,
}

impl ClaudeChat {
    pub fn new<M>(model: M, max_tokens: u64) -> Self
    where
        M: AsRef<str>,
    {
        Self {
            model: model.as_ref().to_string(),
            messages: vec![],
            max_tokens,
            stream: false,
            system: vec![],
            tools: vec![],
        }
    }

    pub fn add_system_promp<S>(&mut self, text: S)
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
        for t in tools.list {
            if t.name != "get_ip_address" {
                continue;
            }

            let claude_tool: ClaudeTool = match t.try_into() {
                Ok(v) => v,
                Err(e) => {
                    error!("{e}");
                    continue;
                }
            };

            self.tools.push(claude_tool);

            break;
        }
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
    pub fn new(config: &ClaudeAiConfig) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            model: config.model.to_string(),
            url: config.url.to_string(),
            anthropic_version: config.anthropic_version.to_string(),
            key: config.key.to_string(),
        })
    }

    pub async fn chat(&self, chat: &ClaudeChat) -> Result<ClaudeResponse> {
        let req_json = serde_json::to_string_pretty(&chat)?;

        let url = format!("{}/v1/messages", self.url);

        let res = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.key)
            .header("anthropic-version", &self.anthropic_version)
            .body(req_json)
            .send()
            .await?;

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

        //fs::write("/tmp/claude_response.json", resp_json.as_bytes())?;

        let resp: ClaudeResponse = match serde_json::from_str(resp_json) {
            Ok(v) => v,
            Err(_) => {
                //
                // Try to print it nicely, best effort
                //
                let claude_error: ClaudeError = serde_json::from_str(resp_json)?;
                let pretty_error = serde_json::to_string_pretty(&claude_error)?;
                let pretty_error = format!("# Claude Error\n\n```json\n{pretty_error}\n```");
                return Err(Error::LlmError { message: pretty_error });
            }
        };

        Ok(resp)
    }

    pub async fn message<S>(&self, content: S) -> Result<ClaudeResponse>
    where
        S: AsRef<str>,
    {
        let mut chat = ClaudeChat::new(&self.model, 4096);

        chat.add_content(ClaudeRole::User, content.as_ref());

        self.chat(&chat).await
    }
}
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
