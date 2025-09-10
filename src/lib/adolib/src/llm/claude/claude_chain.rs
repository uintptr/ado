use crate::{
    config::loader::AdoConfig,
    data::types::AdoData,
    error::Result,
    llm::{
        chain::{LLMChainTrait, LLMUsage},
        claude::{
            claude_api::{
                ClaudeApi, ClaudeContentType, ClaudeMessages, ClaudeResponse, ClaudeRole, ClaudeStopReason,
                ClaudeToolResult,
            },
            claude_config::ClaudeToolChoiceType,
        },
    },
    mcp::matrix::McpMatrix,
    ui::ConsoleDisplayTrait,
};

use async_trait::async_trait;
use log::info;
use omcp::types::McpParams;

pub struct ClaudeChain {
    api: ClaudeApi,
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

        if let Some(tool_choice) = &claude.tool_choice {
            match tool_choice.choice_type {
                ClaudeToolChoiceType::None => {}
                ClaudeToolChoiceType::Any => {
                    // try to load the tools from resources
                    todo!()
                }
            }
        }

        Ok(Self {
            api: ClaudeApi::new(claude)?,
            messages,
            tokens: LLMUsage::default(),
        })
    }

    async fn process_content<C>(&mut self, mcp: &McpMatrix, response: &ClaudeResponse, console: &mut C) -> Result<()>
    where
        C: ConsoleDisplayTrait,
    {
        for content in response.content.iter() {
            match content.content_type {
                ClaudeContentType::Text => {
                    if let Some(text) = &content.text {
                        console.display_string(text)?;
                        self.messages.add_message(ClaudeRole::Assistant, text);
                    }
                }
                ClaudeContentType::ToolUse => {
                    if let Some(name) = &content.name {
                        let mut params = McpParams::new(name);

                        if let Some(input) = &content.input {
                            let args = input.clone();
                            params.set_argument(args);
                        }

                        self.messages.add_content(ClaudeRole::Assistant, &content)?;

                        let (mcp_data, success) = match mcp.call(&params).await {
                            Ok(v) => (v, true),
                            Err(e) => (format!("error: {e}"), false),
                        };

                        let result = ClaudeToolResult::new(&content, mcp_data, success);

                        self.messages.add_result(&result)?;
                    }
                }
                ClaudeContentType::ToolResult => {
                    panic!()
                }
            }
        }

        Ok(())
    }
}

#[async_trait(?Send)]
impl LLMChainTrait for ClaudeChain {
    async fn link<C>(&mut self, mcp: &McpMatrix, content: &str, console: &mut C) -> Result<()>
    where
        C: ConsoleDisplayTrait,
    {
        self.messages.add_message(ClaudeRole::User, content);

        loop {
            let resp = self.api.chat(&self.messages).await?;

            self.tokens.input_tokens += resp.usage.input_tokens;
            self.tokens.output_tokens += resp.usage.output_tokens;

            // in its own function so it can be tested from a local
            // file
            self.process_content(mcp, &resp, console).await?;

            //
            // Keep going until it's done
            //
            if let ClaudeStopReason::EndTurn = resp.stop_reason {
                break;
            } else {
                info!("{} != EndTurn. Continuing", resp.stop_reason);
            }
        }

        Ok(())
    }

    async fn message(&self, content: &str) -> Result<String> {
        let resp = self.api.message(content).await?;
        Ok(resp.message()?.to_string())
    }

    fn reset(&mut self) {
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

    fn enable_mcp(&mut self, mcp: &McpMatrix) -> Result<()> {
        self.messages.with_tools(mcp);
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////
// TEST
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use log::info;
    use rstaples::{logging::StaplesLogger, staples::find_file};

    use crate::{
        config::loader::AdoConfig,
        llm::{
            chain::LLMChainTrait,
            claude::{claude_api::ClaudeResponse, claude_chain::ClaudeChain},
        },
        logging::logger::setup_logger,
        mcp::matrix::McpMatrix,
        ui::NopConsole,
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

        let mut console = NopConsole::new();

        let mcp = McpMatrix::new();

        chain.link(&mcp, "Hello World", &mut console).await.unwrap();
        chain.link(&mcp, "Can you tell a joke", &mut console).await.unwrap();

        chain.message("hello world").await.unwrap();
    }

    #[tokio::test]
    async fn test_content_response() {
        setup_logger(true).unwrap();
        let test_file = Path::new("/tmp").join("claude_response.json");

        let resp = fs::read_to_string(test_file).unwrap();

        let resp: ClaudeResponse = serde_json::from_str(&resp).unwrap();

        let config_file = AdoConfig::from_default().unwrap();
        let mut chain = ClaudeChain::new(&config_file).unwrap();

        let mut console = NopConsole::new();

        let mcp = McpMatrix::new();

        let ret = chain.process_content(&mcp, &resp, &mut console).await.unwrap();

        info!("ret: {ret:?}");
    }

    #[tokio::test]
    async fn test_mcp_response() {
        setup_logger(true).unwrap();

        let config = AdoConfig::from_default().unwrap();
        let mut chain = ClaudeChain::new(&config).unwrap();

        let test_file = Path::new("test").join("claude_mcp_use.json");
        let test_file = find_file(test_file).unwrap();
        let resp_json = fs::read_to_string(test_file).unwrap();
        let resp: ClaudeResponse = serde_json::from_str(&resp_json).unwrap();

        let mut console = NopConsole::new();

        let mut mcp = McpMatrix::new();

        mcp.load(&config).await.unwrap();

        chain.process_content(&mcp, &resp, &mut console).await.unwrap();

        info!("done");
    }
}
