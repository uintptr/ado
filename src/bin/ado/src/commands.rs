use std::{
    fmt::Display,
    io::{self, Write},
};

use adolib::{config::loader::AdoConfig, data::types::AdoData, llm::chain::LLMChain};
use anyhow::{Context, Result};
use log::info;

use crate::intrinsics::IntrinsicPrompts;

pub struct UserCommands {
    chain: LLMChain,
}

pub struct UserCommandEntry {
    pub name: String,
    pub aliases: Vec<String>,
}

fn init_chain(config: &AdoConfig) -> Result<LLMChain> {
    let mut chain = LLMChain::new(config)?;

    for p in IntrinsicPrompts::iter() {
        if let Some(data) = IntrinsicPrompts::get(&p) {
            let prompt = String::from_utf8_lossy(&data.data);
            chain.add_prompt(prompt);
        }
    }

    Ok(chain)
}

fn command_clear() -> Result<()> {
    let mut stdout = io::stdout();
    print!("{esc}c", esc = 27 as char);
    stdout.flush()?;
    Ok(())
}

impl UserCommands {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        let chain = init_chain(config).context("Unable to initialize llm chain")?;

        Ok(Self { chain })
    }

    pub fn command_models(&self) -> Result<()> {
        println!("Models:");
        for m in self.chain.models() {
            println!("* {m}")
        }

        Ok(())
    }

    pub fn handler<C, S>(&mut self, input: S, console: C) -> Result<()>
    where
        C: Fn(AdoData) -> std::result::Result<(), adolib::error::Error> + Send + Sync,
        S: AsRef<str> + Display,
    {
        info!("input: {input}");

        if let Some(command) = input.as_ref().strip_prefix("/") {
            match command {
                "models" => self.command_models()?,
                "clear" | "reset" => command_clear()?,
                _ => println!("Command Not Found ({command})"),
            }
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
