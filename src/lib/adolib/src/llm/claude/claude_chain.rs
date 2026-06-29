use std::{fmt::Display, sync::atomic::AtomicI32};

use log::info;

use crate::{
    config::loader::AdoConfig,
    data::types::AdoData,
    error::{Error, Result},
    llm::{
        chain::{LLMChainTrait, LLMRole, LLMUsage},
        claude::claude_api::{ClaudeApi, ClaudeMessages, ClaudeRole},
    },
};

pub struct ClaudeChain {
    api: ClaudeApi,
    msg_id: AtomicI32,
    messages: ClaudeMessages,
    tokens: LLMUsage,
}

// https://docs.anthropic.com/en/api/messages
// https://docs.anthropic.com/en/docs/agents-and-tools/tool-use/implement-tool-use#example-of-tool-result-with-documents

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////
impl ClaudeChain {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        let claude = config.claude()?;

        let mut messages = ClaudeMessages::new(&claude.model, claude.max_tokens);

        // Constrain responses to the AdoData schema (structured outputs).
        messages.set_output_schema(&crate::data::types::ado_data_schema());

        // if the user defined instructions in the config file
        if let Some(instructions) = &claude.instructions {
            for i in instructions {
                messages.add_system_prompt(i);
            }
        }

        Ok(Self {
            api: ClaudeApi::new(claude),
            msg_id: AtomicI32::new(0),
            messages,
            tokens: LLMUsage::default(),
        })
    }
}

impl LLMChainTrait for ClaudeChain {
    fn call(&mut self) -> Result<AdoData> {
        self.messages.set_cache_breakpoints();

        let resp = self.api.chat(&self.messages)?;

        let usage = &resp.usage;
        info!(
            "tokens: input={} output={} cache_read={} cache_write={}",
            usage.input_tokens,
            usage.output_tokens,
            usage.cache_read_input_tokens,
            usage.cache_creation_input_tokens
        );

        self.tokens.input_tokens = self.tokens.input_tokens.saturating_add(resp.usage.input_tokens);
        self.tokens.output_tokens =
            self.tokens.output_tokens.saturating_add(resp.usage.output_tokens);

        let text = resp.message()?;

        self.messages.add_message(ClaudeRole::Assistant, text);

        let data: AdoData = text.parse()?;

        Ok(data)
    }

    fn models(&self) -> Vec<String> {
        let mut models = Vec::new();

        if let Ok(ret_models) = self.api.models() {
            for model in ret_models {
                models.push(model.id);
            }
        }

        models
    }

    fn add_content<S>(&mut self, role: LLMRole, content: S)
    where
        S: Into<String>,
    {
        match role {
            // System instructions/prompts belong in the cacheable `system`
            // field, not as assistant turns in the message list.
            LLMRole::System => self.messages.add_system_prompt(content.into()),
            LLMRole::Assistant => self.messages.add_message(ClaudeRole::Assistant, content),
            LLMRole::User => self.messages.add_message(ClaudeRole::User, content),
        }
    }

    fn message<S, M>(&self, content: S, _model: Option<M>) -> Result<String>
    where
        S: Into<String>,
        M: AsRef<str>,
    {
        let resp = self.api.message(content)?;
        Ok(resp.message()?.to_string())
    }

    fn reset(&mut self) {
        self.msg_id = AtomicI32::new(0);
        self.tokens = LLMUsage::default();
        self.messages.reset();
    }

    fn model(&self) -> &str {
        &self.api.config.model
    }

    fn change_model<S>(&mut self, model: S) -> Result<()>
    where
        S: AsRef<str> + Display,
    {
        self.api.config.model = model.as_ref().to_string();
        Ok(())
    }

    fn usage(&self) -> LLMUsage {
        LLMUsage {
            input_tokens: self.tokens.input_tokens,
            output_tokens: self.tokens.output_tokens,
        }
    }

    fn dump_chain(&self) -> Result<AdoData> {
        Err(Error::NotImplemented)
    }
}

///////////////////////////////////////////////////////////////////////////////
// TEST
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {

    use crate::{
        config::loader::AdoConfig,
        llm::{chain::LLMChainTrait, claude::claude_chain::ClaudeChain},
    };

    #[test]
    #[ignore = "requires a Claude API key and network access"]
    fn test_message() {
        let config_file = AdoConfig::from_default().unwrap();

        let chain = ClaudeChain::new(&config_file).unwrap();

        chain.message("hello world", None::<&str>).unwrap();
    }
}
