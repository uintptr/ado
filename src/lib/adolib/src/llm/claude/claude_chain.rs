use crate::{
    config::loader::AdoConfig,
    error::Result,
    llm::{
        chain::LLMChainTrait,
        claude::claude_api::{ClaudeApi, ClaudeChat, ClaudeContent, ClaudeContentType, ClaudeRole},
    },
    tools::{handler::ToolHandler, loader::Tools},
    ui::ConsoleDisplayTrait,
};

use async_trait::async_trait;
use log::info;

pub struct ClaudeChain {
    api: ClaudeApi,
    chat: ClaudeChat,
    tool_handler: ToolHandler,
}

impl ClaudeChain {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        let claude = config.claude()?;

        let mut chat = ClaudeChat::new(&claude.model, claude.max_tokens);

        // if the user defined instructions in the config file
        if let Some(instructions) = &claude.instructions {
            for i in instructions {
                chat.add_system_prompt(i);
            }
        }

        // try to load the tools from resources
        let tools = Tools::load()?;

        chat.with_tools(tools);

        Ok(Self {
            api: ClaudeApi::new(claude)?,
            chat,
            tool_handler: ToolHandler::new(config)?,
        })
    }

    async fn process_content<C>(&self, contents: &Vec<ClaudeContent>, console: &mut C) -> Result<()>
    where
        C: ConsoleDisplayTrait,
    {
        for content in contents {
            match content.content_type {
                ClaudeContentType::Text => {
                    if let Some(text) = &content.text {
                        console.display_string(text)?;
                    }
                }
                ClaudeContentType::ToolUse => {
                    if let Some(name) = &content.name {
                        let data = self.tool_handler.call(name, "{}").await?;
                        console.display(data)?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[async_trait(?Send)]
impl LLMChainTrait for ClaudeChain {
    async fn link<C>(&mut self, content: &str, console: &mut C) -> Result<()>
    where
        C: ConsoleDisplayTrait,
    {
        self.chat.add_content(ClaudeRole::User, content);

        let resp = self.api.chat(&self.chat).await?;

        info!("id={}", resp.id);

        // in its own function so it can be tested from a local
        // file
        self.process_content(&resp.content, console).await?;

        let resp_role = resp.role.clone();

        let msg = resp.message()?;

        self.chat.add_content(resp_role, msg);

        Ok(())
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
mod tests {
    use std::{fs, path::Path};

    use log::info;
    use rstaples::logging::StaplesLogger;

    use crate::{
        config::loader::AdoConfig,
        llm::{
            chain::LLMChainTrait,
            claude::{claude_api::ClaudeResponse, claude_chain::ClaudeChain},
        },
        logging::logger::setup_logger,
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

        let console = NopConsole {};

        chain.link("Hello World", &mut console).await.unwrap();
        chain.link("Can you tell a joke", &mut console).await.unwrap();

        chain.message("hello world").await.unwrap();
    }

    #[tokio::test]
    async fn test_content_response() {
        setup_logger(true).unwrap();
        let test_file = Path::new("/tmp").join("claude_response.json");

        let resp = fs::read_to_string(test_file).unwrap();

        let resp: ClaudeResponse = serde_json::from_str(&resp).unwrap();

        let config_file = AdoConfig::from_default().unwrap();
        let chain = ClaudeChain::new(&config_file).unwrap();

        let console = NopConsole {};

        let ret = chain.process_content(&resp.content, &mut console).await.unwrap();

        info!("ret: {ret:?}");
    }
}
