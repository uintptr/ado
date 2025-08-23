#![allow(dead_code)] // message is dead code for native but required for wasm
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    config::loader::AdoConfig,
    data::types::{AdoData, AdoDataMarkdown},
    error::{Error, Result},
    llm::{claude::claude_chain::ClaudeChain, ollama::ollama_chain::OllamaChain, openai::openai_chain::OpenAIChain},
    ui::ConsoleDisplayTrait,
};

use log::error;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LLMUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl AdoDataMarkdown for LLMUsage {
    fn to_markdown(self) -> Result<String> {
        let mut lines = Vec::new();

        lines.push("# LLM Usage".to_string());
        lines.push(format!(" * Input Tokens: {}", self.input_tokens));
        lines.push(format!(" * Ouput Tokens: {}", self.output_tokens));

        let md = lines.join("\n");
        Ok(md)
    }
}

pub enum LLMToolState {
    Enable,
    Disable,
}

#[async_trait(?Send)]
pub trait LLMChainTrait {
    async fn link<C>(&mut self, content: &str, console: &mut C) -> Result<()>
    where
        C: ConsoleDisplayTrait;
    async fn message(&self, content: &str) -> Result<String>;
    fn reset(&mut self);
    fn model(&self) -> &str;
    fn change_model<S: AsRef<str>>(&mut self, _model: S);
    fn usage(&self) -> LLMUsage;
    fn dump_chain(&self) -> Result<AdoData>;
    fn tool(&mut self, _state: LLMToolState) -> Result<()> {
        Err(Error::NotImplemented)
    }
}

pub enum LLMChain {
    OpenAI(Box<OpenAIChain>),
    Ollama(Box<OllamaChain>),
    Claude(Box<ClaudeChain>),
}

impl LLMChain {
    pub fn new(config: &AdoConfig) -> Result<LLMChain> {
        let chain = match config.llm_provider() {
            "openai" => {
                let chain = OpenAIChain::new(config)?;
                LLMChain::OpenAI(Box::new(chain))
            }
            "ollama" => {
                let chain = OllamaChain::new(config)?;
                LLMChain::Ollama(Box::new(chain))
            }
            "claude" => {
                let chain = ClaudeChain::new(config)?;
                LLMChain::Claude(Box::new(chain))
            }
            unk => {
                error!("Unknown provider: {unk}");
                return Err(Error::LlmNotFound { llm: unk.into() });
            }
        };

        Ok(chain)
    }

    pub async fn message(&self, content: &str) -> Result<String> {
        match self {
            LLMChain::OpenAI(openai) => openai.message(content).await,
            LLMChain::Ollama(ollama) => ollama.message(content).await,
            LLMChain::Claude(claude) => claude.message(content).await,
        }
    }

    pub async fn link<C>(&mut self, content: &str, console: &mut C) -> Result<()>
    where
        C: ConsoleDisplayTrait,
    {
        match self {
            LLMChain::OpenAI(openai) => openai.link(content, console).await,
            LLMChain::Ollama(ollama) => ollama.link(content, console).await,
            LLMChain::Claude(claude) => claude.link(content, console).await,
        }
    }

    pub fn reset(&mut self) {
        match self {
            LLMChain::OpenAI(openai) => openai.reset(),
            LLMChain::Ollama(ollama) => ollama.reset(),
            LLMChain::Claude(claude) => claude.reset(),
        }
    }

    pub fn model(&self) -> &str {
        match self {
            LLMChain::OpenAI(openai) => openai.model(),
            LLMChain::Ollama(ollama) => ollama.model(),
            LLMChain::Claude(claude) => claude.model(),
        }
    }

    pub fn change_model<S>(&mut self, model: S)
    where
        S: AsRef<str>,
    {
        match self {
            LLMChain::OpenAI(openai) => openai.change_model(model),
            LLMChain::Ollama(ollama) => ollama.change_model(model),
            LLMChain::Claude(claude) => claude.change_model(model),
        }
    }

    pub fn usage(&self) -> LLMUsage {
        match self {
            LLMChain::OpenAI(openai) => openai.usage(),
            LLMChain::Ollama(ollama) => ollama.usage(),
            LLMChain::Claude(claude) => claude.usage(),
        }
    }

    pub fn dump_chain(&self) -> Result<AdoData> {
        match self {
            LLMChain::OpenAI(openai) => openai.dump_chain(),
            LLMChain::Ollama(ollama) => ollama.dump_chain(),
            LLMChain::Claude(claude) => claude.dump_chain(),
        }
    }

    pub fn tool(&mut self, state: LLMToolState) -> Result<()> {
        match self {
            LLMChain::OpenAI(openai) => openai.tool(state),
            LLMChain::Ollama(ollama) => ollama.tool(state),
            LLMChain::Claude(claude) => claude.tool(state),
        }
    }
}
