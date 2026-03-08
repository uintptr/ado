use std::sync::atomic::AtomicI32;

use crate::{
    config::loader::AdoConfig,
    data::types::AdoData,
    error::Result,
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

        // if the user defined instructions in the config file
        if let Some(instructions) = &claude.instructions {
            for i in instructions {
                messages.add_system_prompt(i);
            }
        }

        Ok(Self {
            api: ClaudeApi::new(claude)?,
            msg_id: AtomicI32::new(0),
            messages,
            tokens: LLMUsage::default(),
        })
    }
}

impl LLMChainTrait for ClaudeChain {
    fn models(&self) -> Vec<String> {
        let mut models = Vec::new();

        if let Ok(ret_models) = self.api.models() {
            for model in ret_models {
                models.push(model.id)
            }
        }

        models
    }

    fn add_content<S>(&mut self, role: LLMRole, content: S)
    where
        S: Into<String>,
    {
        let claude_role = match role {
            LLMRole::Assistant | LLMRole::System => ClaudeRole::Assistant,
            LLMRole::User => ClaudeRole::User,
        };

        self.messages.add_message(claude_role, content)
    }

    fn message<S>(&self, content: S) -> Result<String>
    where
        S: Into<String>,
    {
        let resp = self.api.message(content)?;
        Ok(resp.message()?.to_string())
    }

    fn reset(&mut self) {
        self.msg_id = AtomicI32::new(0);
        self.tokens = LLMUsage::default();
        self.messages.reset()
    }

    fn model(&self) -> &str {
        &self.api.config.model
    }

    fn change_model<S>(&mut self, model: S)
    where
        S: Into<String>,
    {
        self.api.config.model = model.into()
    }

    fn usage(&self) -> LLMUsage {
        LLMUsage {
            input_tokens: self.tokens.input_tokens,
            output_tokens: self.tokens.output_tokens,
        }
    }

    fn dump_chain(&self) -> Result<AdoData> {
        todo!()
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
    fn test_message() {
        let config_file = AdoConfig::from_default().unwrap();

        let chain = ClaudeChain::new(&config_file).unwrap();

        chain.message("hello world").unwrap();
    }
}
