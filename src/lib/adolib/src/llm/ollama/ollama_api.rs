use std::{fmt::Display, vec};

use log::{error, info};
use serde::{Deserialize, Serialize};
use ureq::AsSendBody;

use crate::{
    error::{Error, Result},
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

#[derive(Deserialize)]
pub struct OllamaModel {
    pub name: String,
}

#[derive(Deserialize)]
pub struct OllamaModelResponse {
    pub models: Vec<OllamaModel>,
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
        self.messages = vec![];
    }
}

// https://github.com/ollama/ollama/blob/main/docs/api.md
pub struct OllamaApi {
    pub config: ConfigOllama,
}

impl OllamaApi {
    #[must_use]

    pub fn new(config: &ConfigOllama) -> Self {
        Self { config: config.clone() }
    }

    fn _ollama_delete(&self, url: &str) -> Result<()> {
        let res = ureq::delete(url).call()?;

        let log_msg = format!(
            "get {} -> code={} reason={}",
            url,
            res.status().as_u16(),
            res.status().as_str()
        );

        if res.status().is_success() {
            info!("{log_msg}");
            Ok(())
        } else {
            error!("{log_msg}");
            Err(Error::HttpDeleteFailure)
        }
    }

    fn ollama_post(&self, url: &str, data: impl AsSendBody) -> Result<String> {
        let mut res = ureq::post(url).header("Content-Type", "application/json").send(data)?;

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

        let resp_json = res.body_mut().read_to_string()?;

        Ok(resp_json)
    }

    fn ollama_get(&self, url: &str) -> Result<String> {
        let mut res = ureq::get(url).call()?;

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

        let resp_json = res.body_mut().read_to_string()?;

        Ok(resp_json)
    }

    fn _list_running_models(&self) -> Result<Vec<OllamaModel>> {
        let url = format!("{}/api/ps", self.config.endpoint);

        let resp_json = self.ollama_get(&url)?;

        let models: OllamaModelResponse = serde_json::from_str(&resp_json)?;

        Ok(models.models)
    }

    fn _stop_model(&self, model: &str) -> Result<()> {
        let url = format!("{}/api/models/{model}", self.config.endpoint);
        self._ollama_delete(&url)
    }

    fn _stop_all(&self) -> Result<()> {
        let running_models = self._list_running_models()?;

        for m in running_models {
            if let Err(e) = self._stop_model(&m.name) {
                error!("Unable to stop {}. Error: {e}", m.name);
            }
        }

        Ok(())
    }

    pub fn chat(&self, chat: &OllamaChat) -> Result<OllamaChatResponse> {
        let req_json = serde_json::to_string_pretty(&chat)?;

        let url = format!("{}/api/chat", self.config.endpoint);

        let resp_json = self.ollama_post(&url, &req_json)?;

        let resp: OllamaChatResponse = serde_json::from_str(&resp_json)?;

        Ok(resp)
    }

    pub fn models(&self) -> Result<Vec<OllamaModel>> {
        let url = format!("{}/api/tags", self.config.endpoint);

        let resp_json = self.ollama_get(&url)?;

        let resp: OllamaModelResponse = serde_json::from_str(&resp_json)?;

        Ok(resp.models)
    }

    pub fn message<S>(&self, content: S) -> Result<OllamaChatResponse>
    where
        S: Into<String>,
    {
        let mut chat = OllamaChat::new(&self.config.model);

        chat.add_content(LLMRole::User, content);

        self.chat(&chat)
    }

    pub fn _set_model<S>(&self, _model: S) -> Result<()>
    where
        S: AsRef<str> + Display,
    {
        Ok(())
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
        })
    }

    #[test]
    fn test_set_model() {
        let config = if let Ok(v) = make_config() { v } else { return };
        let api = OllamaApi::new(&config);

        api._set_model("qwen3.5:4B").unwrap();
    }
}
