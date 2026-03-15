use std::{fmt::Display, vec};

use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{
    error::Result,
    llm::{chain::LLMRole, ollama::ollama_config::ConfigOllama},
    rest::{rest_get, rest_post},
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

#[derive(Deserialize)]
pub struct OllamaModel {
    pub name: String,
}

#[derive(Deserialize)]
pub struct OllamaModelResponse {
    pub models: Vec<OllamaModel>,
}

#[derive(Serialize)]
struct OllamaGenerate<'a> {
    model: &'a str,
    keep_alive: i32,
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
    pub model: String,
    messages: Vec<OllamaMessage>,
    think: bool,
    stream: bool,
}

impl OllamaChat {
    pub fn new<M>(model: M, think: bool) -> Self
    where
        M: AsRef<str>,
    {
        info!("thinking: {think}");

        Self {
            model: model.as_ref().to_string(),
            messages: vec![],
            think,
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
        self.messages = vec![];
    }
}

// https://docs.ollama.com/api/generate
pub struct OllamaApi {
    pub config: ConfigOllama,
}

impl OllamaApi {
    pub fn new(config: &ConfigOllama) -> Self {
        Self { config: config.clone() }
    }

    fn list_running_models(&self) -> Result<Vec<OllamaModel>> {
        let url = format!("{}/api/ps", self.config.endpoint);

        let resp_json = rest_get(&url)?;

        let models: OllamaModelResponse = serde_json::from_str(&resp_json)?;

        Ok(models.models)
    }

    fn stop_model(&self, model: &str) -> Result<()> {
        let url = format!("{}/api/generate", self.config.endpoint);

        let request = OllamaGenerate {
            model: model.as_ref(),
            keep_alive: 0,
        };

        rest_post(&url, &request)?;
        Ok(())
    }

    fn stop_all(&self) -> Result<()> {
        let running_models = self.list_running_models()?;

        for m in running_models {
            if let Err(e) = self.stop_model(&m.name) {
                error!("Unable to stop {}. Error: {e}", m.name);
            }
        }

        Ok(())
    }

    fn start_model<S>(&self, model: S) -> Result<()>
    where
        S: AsRef<str> + Display,
    {
        let url = format!("{}/api/generate", self.config.endpoint);

        let request = OllamaGenerate {
            model: model.as_ref(),
            keep_alive: self.config.keep_alive,
        };

        rest_post(&url, &request)?;
        Ok(())
    }

    pub fn chat(&self, chat: &OllamaChat) -> Result<OllamaChatResponse> {
        let url = format!("{}/api/chat", self.config.endpoint);

        let resp_json = rest_post(&url, chat)?;

        let resp: OllamaChatResponse = serde_json::from_str(&resp_json)?;

        Ok(resp)
    }

    pub fn models(&self) -> Result<Vec<OllamaModel>> {
        let url = format!("{}/api/tags", self.config.endpoint);

        let resp_json = rest_get(&url)?;

        let resp: OllamaModelResponse = serde_json::from_str(&resp_json)?;

        Ok(resp.models)
    }

    pub fn message<S>(&self, content: S) -> Result<OllamaChatResponse>
    where
        S: Into<String>,
    {
        let mut chat = OllamaChat::new(&self.config.model, self.config.thinking);

        chat.add_content(LLMRole::User, content);

        self.chat(&chat)
    }

    pub fn set_model<S>(&self, model: S) -> Result<()>
    where
        S: AsRef<str> + Display,
    {
        self.stop_all()?;
        self.start_model(model)
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    fn make_config() -> Result<ConfigOllama> {
        let host = env::var("OLLAMA_HOST")?;

        Ok(ConfigOllama {
            endpoint: host,
            model: "llama3".to_string(),
            thinking: false,
            keep_alive: 30,
        })
    }

    #[test]
    fn test_set_model() {
        let config = if let Ok(v) = make_config() { v } else { return };
        let api = OllamaApi::new(&config);

        let models = api.models().unwrap();

        for m in &models {
            api.set_model(&m.name).unwrap();
        }
    }
}
