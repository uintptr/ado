use std::{
    env,
    fmt::Display,
    fs,
    io::{self, Write},
};

use adolib::{
    config::loader::AdoConfig,
    data::types::AdoData,
    llm::chain::{LLMChain, LLMRole},
};
use anyhow::{Context, Result};
use log::{error, info};

use crate::intrinsics::IntrinsicPrompts;

pub struct UserCommands {
    chain: LLMChain,
}

pub struct UserCommandEntry {
    pub name: String,
    pub aliases: Vec<String>,
}

fn load_intrinsics(chain: &mut LLMChain) {
    for p in IntrinsicPrompts::iter() {
        if let Some(data) = IntrinsicPrompts::get(&p) {
            info!("loading embeded prompt={p}");
            let prompt = String::from_utf8_lossy(&data.data);
            chain.add_content(LLMRole::System, prompt);
        }
    }
}

fn load_ado_md(chain: &mut LLMChain) -> Result<()> {
    let cwd = env::current_dir().context("Unable to get current directory")?;

    let ado_md = cwd.join("ADO.md");

    if ado_md.exists() {
        info!("reading {}", ado_md.display());
        let data = fs::read_to_string(&ado_md).with_context(|| format!("Unable to read {}", ado_md.display()))?;
        chain.add_content(LLMRole::System, data);
    }

    Ok(())
}

fn init_chain(config: &AdoConfig) -> Result<LLMChain> {
    let mut chain = LLMChain::new(config)?;

    load_intrinsics(&mut chain);

    if let Err(e) = load_ado_md(&mut chain) {
        error!("Unable to load ADO.md files ({e})");
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
            self.chain.link(input.as_ref(), console)?;
        }
        Ok(())
    }

    pub fn list_commands(&self) -> Vec<UserCommandEntry> {
        vec![]
    }
}
