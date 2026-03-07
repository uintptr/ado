use std::fmt::Display;

use crate::{
    config::loader::AdoConfig,
    data::types::AdoData,
    error::Result,
    llm::{
        chain::{LLMChainTrait, LLMUsage},
        ollama::ollama_api::{OllamaApi, OllamaChat},
    },
};

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

impl LLMChainTrait for OllamaChain {
    fn link<C, S>(&mut self, content: S, console: C) -> Result<()>
    where
        C: Fn(AdoData) -> Result<()> + Send + Sync,
        S: AsRef<str> + Display,
    {
        self.chat.add_content("user", content);

        let resp = self.api.chat(&self.chat)?;

        let resp_str = resp.message.content.to_string();

        self.chat.add_message(resp.message);

        console(AdoData::String(resp_str))
    }

    fn message<S>(&self, content: S) -> Result<String>
    where
        S: AsRef<str> + Display,
    {
        let resp = self.api.message(content)?;
        Ok(resp.message.content)
    }

    fn reset(&mut self) {
        self.chat.reset()
    }

    fn model(&self) -> &str {
        &self.api.config.model
    }

    fn change_model<S>(&mut self, model: S)
    where
        S: AsRef<str>,
    {
        self.api.config.model = model.as_ref().into()
    }

    fn usage(&self) -> LLMUsage {
        LLMUsage {
            input_tokens: 0,
            output_tokens: 0,
        }
    }

    fn dump_chain(&self) -> Result<AdoData> {
        unimplemented!()
    }
}

#[cfg(test)]
mod ollama_tests {

    use crate::{
        config::loader::AdoConfig,
        data::types::AdoData,
        error::Result,
        llm::{chain::LLMChainTrait, ollama::ollama_chain::OllamaChain},
    };

    pub fn nop_console(_data: AdoData) -> Result<()> {
        Ok(())
    }

    #[test]
    fn test_message() {
        let config_file = AdoConfig::from_default().unwrap();

        let chain = OllamaChain::new(&config_file).unwrap();

        chain.message("hello world").unwrap();
    }

    #[test]
    fn test_chain() {
        let config_file = AdoConfig::from_default().unwrap();

        let mut chain = OllamaChain::new(&config_file).unwrap();

        chain.link("Hello World", nop_console).unwrap();
        chain.link("Can you tell a joke", nop_console).unwrap();

        chain.message("hello world").unwrap();
    }
}
