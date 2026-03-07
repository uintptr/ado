use std::fmt::Display;

use adolib::{config::loader::AdoConfig, data::types::AdoData, llm::chain::LLMChain};
use anyhow::Result;
use log::info;

pub struct UserCommands {
    chain: LLMChain,
}

pub struct UserCommandEntry {
    pub name: String,
    pub aliases: Vec<String>,
}

impl UserCommands {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        let chain = LLMChain::new(config)?;

        Ok(Self { chain })
    }

    pub fn handler<C, S>(&mut self, input: S, console: C) -> Result<()>
    where
        C: Fn(AdoData) -> std::result::Result<(), adolib::error::Error> + Send + Sync,
        S: AsRef<str> + Display,
    {
        info!("input: {input}");

        if let Some(command) = input.as_ref().strip_prefix("/") {
            info!("command: {command}");
        } else {
            //
            // forward to
            //
            self.chain.link(input, console)?;
        }
        Ok(())
    }

    pub fn list_commands(&self) -> Vec<UserCommandEntry> {
        vec![]
    }
}
