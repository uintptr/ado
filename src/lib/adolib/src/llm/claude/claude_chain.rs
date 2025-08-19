use crate::{
    config::loader::AdoConfig,
    data::types::AdoData,
    error::Result,
    llm::{
        claude::claude_api::{ClaudeApi, ClaudeChat},
        provider::LLMChainTrait,
    },
};

use async_trait::async_trait;

pub struct ClaudeChain {
    api: ClaudeApi,
    chat: ClaudeChat,
}

impl ClaudeChain {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        let claude = config.claude()?;

        let mut chat = ClaudeChat::new(&claude.model, claude.max_tokens);

        if let Some(instructions) = &claude.instructions {
            for i in instructions {
                chat.add_content("user", i);
            }
        }

        Ok(Self {
            api: ClaudeApi::new(claude)?,
            chat,
        })
    }
}

#[async_trait(?Send)]
impl LLMChainTrait for ClaudeChain {
    async fn query(&mut self, content: &str) -> Result<AdoData> {
        self.chat.add_content("user", content);

        let resp = self.api.chat(&self.chat).await?;

        let msg = resp.message()?;

        self.chat.add_content("assistant", msg);

        Ok(AdoData::String(msg.to_string()))
    }

    async fn message(&self, content: &str) -> Result<String> {
        let resp = self.api.message(content).await?;
        Ok(resp.message()?.to_string())
    }

    fn reset(&mut self) {
        self.chat.reset()
    }

    fn model(&self) -> &str {
        &self.api.model
    }
}

#[cfg(test)]
mod ollama_tests {
    use rstaples::logging::StaplesLogger;

    use crate::{
        config::loader::AdoConfig,
        llm::{claude::claude_chain::ClaudeChain, provider::LLMChainTrait},
    };

    #[tokio::test]
    async fn test_message() {
        StaplesLogger::new().with_stdout().start().unwrap();

        let config_file = AdoConfig::from_default().unwrap();

        let chain = ClaudeChain::new(&config_file).unwrap();

        chain.message("hello world").await.unwrap();
    }

    #[tokio::test]
    async fn test_chain() {
        StaplesLogger::new().with_stdout().start().unwrap();

        let config_file = AdoConfig::from_default().unwrap();

        let mut chain = ClaudeChain::new(&config_file).unwrap();

        chain.query("Hello World").await.unwrap();
        chain.query("Can you tell a joke").await.unwrap();

        chain.message("hello world").await.unwrap();
    }
}
