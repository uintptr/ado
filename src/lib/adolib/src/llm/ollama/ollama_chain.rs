use crate::{
    config::loader::AdoConfig,
    data::types::AdoData,
    error::Result,
    llm::{
        chain::LLMChainTrait,
        ollama::ollama_api::{OllamaApi, OllamaChat},
    },
    ui::ConsoleDisplayTrait,
};

use async_trait::async_trait;

pub struct OllamaChain {
    api: OllamaApi,
    chat: OllamaChat,
}

impl OllamaChain {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        let ollama = config.ollama()?;

        Ok(Self {
            api: OllamaApi::new(ollama)?,
            chat: OllamaChat::new(&ollama.model),
        })
    }
}

#[async_trait(?Send)]
impl LLMChainTrait for OllamaChain {
    async fn link<C>(&mut self, content: &str, console: &mut C) -> Result<()>
    where
        C: ConsoleDisplayTrait,
    {
        self.chat.add_content("user", content);

        let resp = self.api.chat(&self.chat).await?;

        let resp_str = resp.message.content.to_string();

        self.chat.add_message(resp.message);

        console.display(AdoData::String(resp_str))
    }

    async fn message(&self, content: &str) -> Result<String> {
        let resp = self.api.message(content).await?;
        Ok(resp.message.content)
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
        llm::{chain::LLMChainTrait, ollama::ollama_chain::OllamaChain},
        ui::NopConsole,
    };

    #[tokio::test]
    async fn test_message() {
        StaplesLogger::new().with_stdout().start().unwrap();

        let config_file = AdoConfig::from_default().unwrap();

        let chain = OllamaChain::new(&config_file).unwrap();

        chain.message("hello world").await.unwrap();
    }

    #[tokio::test]
    async fn test_chain() {
        StaplesLogger::new().with_stdout().start().unwrap();

        let config_file = AdoConfig::from_default().unwrap();

        let mut chain = OllamaChain::new(&config_file).unwrap();

        let console = NopConsole {};

        chain.link("Hello World", &console).await.unwrap();
        chain.link("Can you tell a joke", &console).await.unwrap();

        chain.message("hello world").await.unwrap();
    }
}
