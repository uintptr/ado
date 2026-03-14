use std::fmt::Display;

use crate::{
    config::loader::AdoConfig,
    data::types::AdoData,
    error::{Error, Result},
    llm::{
        chain::{LLMChainTrait, LLMRole, LLMUsage},
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
            api: OllamaApi::new(ollama),
            chat: OllamaChat::new(&ollama.model),
        })
    }
}

impl LLMChainTrait for OllamaChain {
    fn call(&mut self) -> Result<AdoData> {
        let resp = self.api.chat(&self.chat)?;

        self.chat.add_content(LLMRole::Assistant, &resp.message.content);

        let data: AdoData = resp.message.content.parse()?;

        Ok(data)
    }

    fn models(&self) -> Vec<String> {
        let mut names = Vec::new();

        if let Ok(ollama_models) = self.api.models() {
            for model in ollama_models {
                names.push(model.name);
            }
        }
        names
    }

    fn add_content<S>(&mut self, role: LLMRole, content: S)
    where
        S: Into<String>,
    {
        self.chat.add_content(role, content);
    }

    fn message<S>(&self, content: S) -> Result<String>
    where
        S: Into<String>,
    {
        let resp = self.api.message(content)?;
        Ok(resp.message.content)
    }

    fn reset(&mut self) {
        self.chat.reset();
    }

    fn model(&self) -> &str {
        &self.api.config.model
    }

    fn change_model<S>(&mut self, model: S) -> Result<()>
    where
        S: AsRef<str> + Display,
    {
        //self.api.set_model(&model)?;
        self.api.config.model = model.as_ref().to_string();

        Ok(())
    }

    fn usage(&self) -> LLMUsage {
        LLMUsage {
            input_tokens: 0,
            output_tokens: 0,
        }
    }

    fn dump_chain(&self) -> Result<AdoData> {
        Err(Error::NotImplemented)
    }
}

#[cfg(test)]
mod ollama_tests {

    use crate::{
        config::loader::AdoConfig,
        llm::{chain::LLMChainTrait, ollama::ollama_chain::OllamaChain},
    };

    #[test]
    fn test_message() {
        let config_file = AdoConfig::from_default().unwrap();

        let chain = OllamaChain::new(&config_file).unwrap();

        chain.message("hello world").unwrap();
    }
}
