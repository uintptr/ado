#![allow(dead_code)] // message is dead code for native but required for wasm
use async_trait::async_trait;

use crate::{
    config::loader::AdoConfig,
    error::{Error, Result},
    llm::{claude::claude_chain::ClaudeChain, ollama::ollama_chain::OllamaChain, openai::openai_chain::OpenAIChain},
    ui::ConsoleDisplayTrait,
};

use log::error;

#[async_trait(?Send)]
pub trait LLMChainTrait {
    async fn link<C>(&mut self, content: &str, console: &mut C) -> Result<()>
    where
        C: ConsoleDisplayTrait;
    async fn message(&self, content: &str) -> Result<String>;
    fn reset(&mut self);
    fn model(&self) -> &str;
    fn change_model<S: AsRef<str>>(&mut self, _model: S);
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
}
