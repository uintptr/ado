use std::sync::atomic::{AtomicI32, Ordering};

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

use async_trait::async_trait;
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

    async fn process_content<C>(&mut self, response: &ClaudeResponse, console: C) -> Result<()>
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

#[async_trait(?Send)]
impl LLMChainTrait for ClaudeChain {
    async fn link<C>(&mut self, content: &str, console: C) -> Result<()>
    where
        C: Fn(AdoData) -> Result<()> + Send + Sync,
    {
        self.messages.add_message(ClaudeRole::User, content);

        loop {
            let resp = self.api.chat(&self.messages).await?;

            self.tokens.input_tokens += resp.usage.input_tokens;
            self.tokens.output_tokens += resp.usage.output_tokens;

            // in its own function so it can be tested from a local
            // file
            self.process_content(&resp, &console).await?;

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

    async fn message(&self, content: &str) -> Result<String> {
        let resp = self.api.message(content).await?;
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
    use rstaples::{file::find_file, logging::StaplesLogger};

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

    #[tokio::test]
    async fn test_message() {
        StaplesLogger::new().with_stdout().start();

        let config_file = AdoConfig::from_default().unwrap();

        let chain = ClaudeChain::new(&config_file).unwrap();

        chain.message("hello world").await.unwrap();
    }

    #[tokio::test]
    async fn test_chain() {
        StaplesLogger::new().with_stdout().start();

        let config_file = AdoConfig::from_default().unwrap();

        let mut chain = ClaudeChain::new(&config_file).unwrap();

        chain.link("Hello World", nop_console).await.unwrap();
        chain.link("Can you tell a joke", nop_console).await.unwrap();

        chain.message("hello world").await.unwrap();
    }

    #[tokio::test]
    async fn test_content_response() {
        let test_file = Path::new("/tmp").join("claude_response.json");

        let resp = fs::read_to_string(test_file).unwrap();

        let resp: ClaudeResponse = serde_json::from_str(&resp).unwrap();

        let config_file = AdoConfig::from_default().unwrap();
        let mut chain = ClaudeChain::new(&config_file).unwrap();

        let ret = chain.process_content(&resp, nop_console).await.unwrap();

        info!("ret: {ret:?}");
    }

    #[tokio::test]
    async fn test_mcp_response() {
        let config = AdoConfig::from_default().unwrap();
        let mut chain = ClaudeChain::new(&config).unwrap();

        let test_file = Path::new("test").join("claude_mcp_use.json");
        let test_file = find_file(test_file).unwrap();
        let resp_json = fs::read_to_string(test_file).unwrap();
        let resp: ClaudeResponse = serde_json::from_str(&resp_json).unwrap();

        chain.process_content(&resp, nop_console).await.unwrap();

        info!("done");
    }
}
