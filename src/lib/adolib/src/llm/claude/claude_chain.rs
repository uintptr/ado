use std::{
    path::PathBuf,
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
            ClaudeToolResult,
        },
    },
    mcp::matrix::McpMatrix,
    ui::ConsoleDisplayTrait,
};

use async_trait::async_trait;
use log::{error, info};
use omcp::types::McpParams;
use tokio::fs;
use uuid::Uuid;

pub struct ClaudeChain {
    api: ClaudeApi,
    msg_id: AtomicI32,
    log_dir: Option<PathBuf>,
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

        let log_dir = match &claude.logs {
            Some(v) => match shellexpand::full(&v) {
                Ok(v) => {
                    let uuid = Uuid::new_v4().to_string();
                    let log_dir = PathBuf::from(v.to_string()).join(uuid);

                    match std::fs::create_dir_all(&log_dir) {
                        Ok(_) => Some(log_dir),
                        Err(e) => {
                            error!("{e}");
                            None
                        }
                    }
                }
                Err(e) => {
                    error!("{e}");
                    None
                }
            },
            None => None,
        };

        Ok(Self {
            api: ClaudeApi::new(claude)?,
            msg_id: AtomicI32::new(0),
            log_dir,
            messages,
            tokens: LLMUsage::default(),
        })
    }

    async fn process_content<C>(&mut self, mcp: &McpMatrix, response: &ClaudeResponse, console: &mut C) -> Result<()>
    where
        C: ConsoleDisplayTrait,
    {
        let mut sub_id = 0;
        let msg_id = self.msg_id.load(Ordering::SeqCst);

        for content in response.content.iter() {
            //
            // Log response if needed
            //
            if let Some(log_dir) = &self.log_dir {
                let file_name = format!("{msg_id:04}_{sub_id:04}_{}.json", response.id);
                let file_path = log_dir.join(file_name);
                sub_id += 1;

                if let Ok(response_json) = serde_json::to_string_pretty(&response)
                    && let Err(e) = fs::write(file_path, response_json.as_bytes()).await
                {
                    error!("{e}");
                }
            }

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

                        self.messages.add_content(ClaudeRole::Assistant, content)?;

                        let (mcp_data, success) = match mcp.call(&params).await {
                            Ok(v) => (v, true),
                            Err(e) => (format!("error: {e}"), false),
                        };

                        let result = ClaudeToolResult::new(content, mcp_data, success);

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

        if let Some(log_dir) = &self.log_dir {
            let msg_id = self.msg_id.load(Ordering::SeqCst);
            let file_name = format!("{msg_id:04}.json");
            let file_path = log_dir.join(file_name);

            if let Ok(messages) = serde_json::to_string_pretty(&self.messages)
                && let Err(e) = fs::write(file_path, messages.as_bytes()).await
            {
                error!("{e}");
            }
        }

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

        mcp.load(&config, "ha").await.unwrap();

        chain.process_content(&mcp, &resp, &mut console).await.unwrap();

        info!("done");
    }
}
