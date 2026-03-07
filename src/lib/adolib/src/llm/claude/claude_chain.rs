use std::{
    fmt::Display,
    sync::atomic::{AtomicI32, Ordering},
};

use crate::{
    config::loader::AdoConfig,
    data::types::AdoData,
    error::Result,
    llm::{
        chain::{LLMChainTrait, LLMUsage},
        claude::claude_api::{
            ClaudeApi, ClaudeContentType, ClaudeMessages, ClaudeResponse, ClaudeRole, ClaudeStopReason,
        },
    },
};

use log::info;

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

    fn process_content<C>(&mut self, response: &ClaudeResponse, console: C) -> Result<()>
    where
        C: Fn(AdoData) -> Result<()> + Send + Sync,
    {
        for content in response.content.iter() {
            match content.content_type {
                ClaudeContentType::Text => {
                    if let Some(text) = &content.text {
                        console(AdoData::String(text.clone()))?;
                        self.messages.add_message(ClaudeRole::Assistant, text);
                    }
                }
            }
        }

        Ok(())
    }
}

impl LLMChainTrait for ClaudeChain {
    fn link<C, S>(&mut self, content: S, console: C) -> Result<()>
    where
        C: Fn(AdoData) -> Result<()> + Send + Sync,
        S: AsRef<str> + Display,
    {
        self.messages.add_message(ClaudeRole::User, content);

        loop {
            let resp = self.api.chat(&self.messages)?;

            self.tokens.input_tokens += resp.usage.input_tokens;
            self.tokens.output_tokens += resp.usage.output_tokens;

            // in its own function so it can be tested from a local
            // file
            self.process_content(&resp, &console)?;

            //
            // Keep going until it's done
            //
            if let ClaudeStopReason::EndTurn = resp.stop_reason {
                break;
            } else {
                info!("{} != EndTurn. Continuing", resp.stop_reason);
            }
        }

        self.msg_id.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    fn message<S>(&self, content: S) -> Result<String>
    where
        S: AsRef<str> + Display,
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
        S: AsRef<str>,
    {
        self.api.config.model = model.as_ref().into()
    }

    fn usage(&self) -> LLMUsage {
        LLMUsage {
            input_tokens: self.tokens.input_tokens,
            output_tokens: self.tokens.output_tokens,
        }
    }

    fn dump_chain(&self) -> Result<AdoData> {
        let json_chain = serde_json::to_string_pretty(&self.messages)?;
        Ok(AdoData::Json(json_chain))
    }
}

///////////////////////////////////////////////////////////////////////////////
// TEST
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use log::info;

    use crate::{
        config::loader::AdoConfig,
        data::types::AdoData,
        error::Result,
        llm::{
            chain::LLMChainTrait,
            claude::{claude_api::ClaudeResponse, claude_chain::ClaudeChain},
        },
    };

    fn nop_console(_data: AdoData) -> Result<()> {
        Ok(())
    }

    #[test]
    fn test_message() {
        let config_file = AdoConfig::from_default().unwrap();

        let chain = ClaudeChain::new(&config_file).unwrap();

        chain.message("hello world").unwrap();
    }

    #[test]
    fn test_chain() {
        let config_file = AdoConfig::from_default().unwrap();

        let mut chain = ClaudeChain::new(&config_file).unwrap();

        chain.link("Hello World", nop_console).unwrap();
        chain.link("Can you tell a joke", nop_console).unwrap();

        chain.message("hello world").unwrap();
    }

    #[test]
    fn test_content_response() {
        let test_file = Path::new("/tmp").join("claude_response.json");

        let resp = fs::read_to_string(test_file).unwrap();

        let resp: ClaudeResponse = serde_json::from_str(&resp).unwrap();

        let config_file = AdoConfig::from_default().unwrap();
        let mut chain = ClaudeChain::new(&config_file).unwrap();

        let ret = chain.process_content(&resp, nop_console).unwrap();

        info!("ret: {ret:?}");
    }
}
