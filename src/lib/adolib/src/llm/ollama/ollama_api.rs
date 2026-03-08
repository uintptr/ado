use std::vec;

use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{
    error::Result,
    llm::{chain::LLMRole, ollama::ollama_config::ConfigOllama},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

impl OllamaMessage {
    pub fn new<R, C>(role: R, content: C) -> Self
    where
        R: Into<String>,
        C: Into<String>,
    {
        Self {
            role: role.into(),
            content: content.into(),
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

    pub fn add_content<C>(&mut self, role: LLMRole, content: C)
    where
        C: Into<String>,
    {
        let ollama_role: String = role.into();

        let message = OllamaMessage::new(ollama_role, content);
        self.messages.push(message);
    }

    pub fn reset(&mut self) {
        self.messages = vec![]
    }
}

// https://github.com/ollama/ollama/blob/main/docs/api.md
pub struct OllamaApi {
    pub config: ConfigOllama,
}

impl OllamaApi {
    pub fn new(config: &ConfigOllama) -> Result<Self> {
        Ok(Self { config: config.clone() })
    }

    pub fn chat(&self, chat: &OllamaChat) -> Result<OllamaChatResponse> {
        let req_json = serde_json::to_string_pretty(&chat)?;

        let url = format!("{}/api/chat", self.config.endpoint);

        let mut res = ureq::post(&url).header("Content-Type", "application/json").send(&req_json)?;

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

        let resp_json = res.body_mut().read_to_string()?;

        let resp: OllamaChatResponse = serde_json::from_str(&resp_json)?;

        Ok(resp)
    }

    pub fn message<S>(&self, content: S) -> Result<OllamaChatResponse>
    where
        S: Into<String>,
    {
        let mut chat = OllamaChat::new(&self.config.model);

        chat.add_content(LLMRole::User, content);

        self.chat(&chat)
    }
}
