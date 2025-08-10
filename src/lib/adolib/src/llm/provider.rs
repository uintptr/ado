#![allow(dead_code)] // message is dead code for native but required for wasm
use async_trait::async_trait;

use crate::{
    config_file::loader::ConfigFile,
    data::types::AdoData,
    error::{Error, Result},
    llm::openai::chain::OpenAIChain,
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
    OpenAI(OpenAIChain),
}

impl LLMChain {
    pub fn new(config: &ConfigFile) -> Result<LLMChain> {
        let chain = match config.llm_provider() {
            "openai" => {
                let chain = OpenAIChain::new(config)?;
                LLMChain::OpenAI(chain)
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
        }
    }

    pub async fn query(&mut self, content: &str) -> Result<AdoData> {
        match self {
            LLMChain::OpenAI(openai) => openai.query(content).await,
        }
    }

    pub fn reset(&mut self) {
        match self {
            LLMChain::OpenAI(openai) => openai.reset(),
        }
    }

    pub fn model(&self) -> &str {
        match self {
            LLMChain::OpenAI(openai) => openai.model(),
        }
    }
}
