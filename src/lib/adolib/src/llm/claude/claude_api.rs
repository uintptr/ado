use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::{config::loader::ClaudeAiConfig, error::Result};

#[derive(Debug, Deserialize, Serialize)]
pub struct ClaudeMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClaudeContent {
    pub text: String,
    #[serde(rename = "type")]
    text_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClaudeUsage {
    input_tokens: u64,
    output_tokens: u64,
}

impl ClaudeMessage {
    pub fn new<R, C>(role: R, content: C) -> Self
    where
        R: AsRef<str>,
        C: AsRef<str>,
    {
        Self {
            role: role.as_ref().to_string(),
            content: content.as_ref().to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ClaudeResponse {
    pub content: Vec<ClaudeContent>,
    //pub id: String,
    //pub model: String,
    //pub role: String,
    //pub stop_reason: String,
    //pub stop_sequence: Option<String>,
    //#[serde(rename = "type")]
    //pub response_type: String,
    //pub usage: ClaudeUsage,
}

impl ClaudeResponse {
    pub fn message(&self) -> Result<&str> {
        let text = self.content.first().ok_or(Error::Empty)?;
        Ok(&text.text)
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
        }
    }

    pub fn add_content<R, C>(&mut self, role: R, content: C)
    where
        R: AsRef<str>,
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

        let resp: ClaudeResponse = serde_json::from_str(resp_json)?;

        Ok(resp)
    }

    pub async fn message<S>(&self, content: S) -> Result<ClaudeResponse>
    where
        S: AsRef<str>,
    {
        let mut chat = ClaudeChat::new(&self.model, 4096);

        chat.add_content("user", content.as_ref());

        self.chat(&chat).await
    }
}
