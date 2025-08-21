use std::vec;

use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{error::Result, llm::ollama::ollama_config::ConfigOllama};

#[derive(Debug, Deserialize, Serialize)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

impl OllamaMessage {
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
pub struct OllamaChatResponse {
    //model: String,
    //created_at: String,
    pub message: OllamaMessage,
    //done_reason: String,
    //done: bool,
    //total_duration: u64,
    //load_duration: u64,
    //prompt_eval_count: i32,
    //prompt_eval_duration: u64,
    //eval_count: i32,
    //eval_duration: u64,
}

#[derive(Debug, Serialize)]
pub struct OllamaChat {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

impl OllamaChat {
    pub fn new<M>(model: M) -> Self
    where
        M: AsRef<str>,
    {
        Self {
            model: model.as_ref().to_string(),
            messages: vec![],
            stream: false,
        }
    }

    pub fn add_content<R, C>(&mut self, role: R, content: C)
    where
        R: AsRef<str>,
        C: AsRef<str>,
    {
        let message = OllamaMessage::new(role, content);
        self.messages.push(message);
    }

    pub fn add_message(&mut self, message: OllamaMessage) {
        self.messages.push(message)
    }

    pub fn reset(&mut self) {
        self.messages = vec![]
    }
}

// https://github.com/ollama/ollama/blob/main/docs/api.md
pub struct OllamaApi {
    client: Client,
    pub config: ConfigOllama,
}

impl OllamaApi {
    pub fn new(config: &ConfigOllama) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            config: config.clone(),
        })
    }

    pub async fn chat(&self, chat: &OllamaChat) -> Result<OllamaChatResponse> {
        let req_json = serde_json::to_string_pretty(&chat)?;

        let url = format!("{}/api/chat", self.config.endpoint);

        let res = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
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

        let resp_json = res.text().await?;

        let resp: OllamaChatResponse = serde_json::from_str(&resp_json)?;

        Ok(resp)
    }

    pub async fn message<S>(&self, content: S) -> Result<OllamaChatResponse>
    where
        S: AsRef<str>,
    {
        let mut chat = OllamaChat::new(&self.config.model);

        chat.add_content("user", content.as_ref());

        self.chat(&chat).await
    }
}
