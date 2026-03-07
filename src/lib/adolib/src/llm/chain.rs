use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
    config::loader::AdoConfig,
    data::types::AdoData,
    error::{Error, Result},
    llm::{claude::claude_chain::ClaudeChain, ollama::ollama_chain::OllamaChain},
};

use log::error;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LLMUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

pub enum LLMToolState {
    Enable,
    Disable,
}

pub trait LLMChainTrait {
    fn add_prompt<P>(&mut self, _prompt: P)
    where
        P: AsRef<str> + Display;
    fn link<C, S>(&mut self, content: S, console: C) -> Result<()>
    where
        C: Fn(AdoData) -> Result<()> + Send + Sync,
        S: AsRef<str> + Display;
    fn message<S>(&self, content: S) -> Result<String>
    where
        S: AsRef<str> + Display;
    fn reset(&mut self);
    fn models(&self) -> Vec<String>;
    fn model(&self) -> &str;
    fn change_model<S>(&mut self, _model: S)
    where
        S: AsRef<str>;
    fn usage(&self) -> LLMUsage;
    fn dump_chain(&self) -> Result<AdoData>;
}

pub enum LLMChain {
    Ollama(Box<OllamaChain>),
    Claude(Box<ClaudeChain>),
}

impl LLMChain {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        let chain = match config.llm_provider() {
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

    pub fn models(&self) -> Vec<String> {
        match self {
            LLMChain::Claude(claude) => claude.models(),
            LLMChain::Ollama(ollama) => ollama.models(),
        }
    }

    pub fn add_prompt<P>(&mut self, prompt: P)
    where
        P: AsRef<str> + Display,
    {
        match self {
            LLMChain::Claude(claude) => claude.add_prompt(prompt),
            LLMChain::Ollama(ollama) => ollama.add_prompt(prompt),
        }
    }

    pub fn message<S>(&self, content: S) -> Result<String>
    where
        S: AsRef<str> + Display,
    {
        match self {
            LLMChain::Ollama(ollama) => ollama.message(content),
            LLMChain::Claude(claude) => claude.message(content),
        }
    }

    pub fn link<C, S>(&mut self, content: S, console: C) -> Result<()>
    where
        C: Fn(AdoData) -> Result<()> + Send + Sync,
        S: AsRef<str> + Display,
    {
        match self {
            LLMChain::Ollama(ollama) => ollama.link(content, console),
            LLMChain::Claude(claude) => claude.link(content, console),
        }
    }

    pub fn reset(&mut self) {
        match self {
            LLMChain::Ollama(ollama) => ollama.reset(),
            LLMChain::Claude(claude) => claude.reset(),
        }
    }

    pub fn model(&self) -> &str {
        match self {
            LLMChain::Ollama(ollama) => ollama.model(),
            LLMChain::Claude(claude) => claude.model(),
        }
    }

    pub fn change_model<S>(&mut self, model: S)
    where
        S: AsRef<str>,
    {
        match self {
            LLMChain::Ollama(ollama) => ollama.change_model(model),
            LLMChain::Claude(claude) => claude.change_model(model),
        }
    }

    pub fn usage(&self) -> LLMUsage {
        match self {
            LLMChain::Ollama(ollama) => ollama.usage(),
            LLMChain::Claude(claude) => claude.usage(),
        }
    }

    pub fn dump_chain(&self) -> Result<AdoData> {
        match self {
            LLMChain::Ollama(ollama) => ollama.dump_chain(),
            LLMChain::Claude(claude) => claude.dump_chain(),
        }
    }
}
