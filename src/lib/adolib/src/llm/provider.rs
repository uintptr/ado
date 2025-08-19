#![allow(dead_code)] // message is dead code for native but required for wasm
use async_trait::async_trait;

use crate::{
    config::loader::AdoConfig,
    data::types::AdoData,
    error::{Error, Result},
    llm::{claude::claude_chain::ClaudeChain, ollama::ollama_chain::OllamaChain, openai::chain::OpenAIChain},
};

use log::error;

#[async_trait(?Send)]
pub trait LLMChainTrait {
    async fn query(&mut self, content: &str) -> Result<AdoData>;
    async fn message(&self, content: &str) -> Result<String>;
    fn reset(&mut self);
    fn model(&self) -> &str;
}

pub enum LLMChain {
    OpenAI(Box<OpenAIChain>), // box because it gros the union/enum unnecessarily
    Ollama(OllamaChain),
    Claude(ClaudeChain),
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
                LLMChain::Ollama(chain)
            }
            "claude" => {
                let chain = ClaudeChain::new(config)?;
                LLMChain::Claude(chain)
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

    pub async fn query(&mut self, content: &str) -> Result<AdoData> {
        match self {
            LLMChain::OpenAI(openai) => openai.query(content).await,
            LLMChain::Ollama(ollama) => ollama.query(content).await,
            LLMChain::Claude(claude) => claude.query(content).await,
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
}
