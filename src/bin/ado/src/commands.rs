use std::{
    env::{self, consts::OS},
    fmt::Display,
    fs,
    io::{self, Write},
    path::Path,
};

use adolib::{
    config::loader::AdoConfig,
    console::ConsoleTrait,
    llm::chain::{LLMChain, LLMRole},
};
use anyhow::{Context, Result, bail};
use log::{error, info};
use ratatui::style::Stylize;

use crate::intrinsics::IntrinsicPrompts;

pub struct UserCommands {
    chain: LLMChain,
}

pub enum UserCommand {
    Help(String),
    Model(String),
    Models(String),
}

impl Display for UserCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            UserCommand::Help(_) => "help",
            UserCommand::Model(_) => "model",
            UserCommand::Models(_) => "models",
        };
        write!(f, "{s}")
    }
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

fn load_useful(chain: &mut LLMChain) -> Result<()> {
    if let Ok(cwd) = env::current_dir() {
        let cwd_prompt = format!("The current working directory is {}", cwd.display());
        chain.add_content(LLMRole::User, cwd_prompt)
    }

    let current_os = format!("The current operating system is {}", OS);
    chain.add_content(LLMRole::User, current_os);

    Ok(())
}

fn load_skills_from_path(chain: &mut LLMChain, path: &Path) -> Result<()> {
    info!("Loading skills from {}", path.display());

    let patt = format!("{}/*.md", path.display());

    for f in glob::glob(&patt)? {
        if let Ok(md_file) = f {
            if let Ok(data) = fs::read_to_string(md_file) {
                chain.add_content(LLMRole::System, data)
            }
        }
    }

    Ok(())
}

fn load_skills(chain: &mut LLMChain) -> Result<()> {
    let mut skills_dirs = Vec::new();

    //
    // Load skills in the user directory
    //
    if let Some(config_dir) = dirs::config_dir() {
        let config_dir = config_dir.join("ado");
        let skills_dir = config_dir.join("skills");

        skills_dirs.push(skills_dir);
    }

    //
    // Load skill in the current directory
    //
    if let Ok(cwd) = env::current_dir() {
        let skills_dir = cwd.join("skills");
        skills_dirs.push(skills_dir);
    }

    for dir in skills_dirs {
        if !dir.exists() {
            info!("{} doesn't exist", dir.display());
            continue;
        }

        if let Err(e) = load_skills_from_path(chain, &dir) {
            error!("Unable to load skills in {} ({e})", dir.display());
        }
    }

    Ok(())
}

fn init_chain(config: &AdoConfig) -> Result<LLMChain> {
    let mut chain = LLMChain::new(config)?;

    load_intrinsics(&mut chain);

    if let Err(e) = load_ado_md(&mut chain) {
        error!("Unable to load ADO.md files ({e})");
    }

    if let Err(e) = load_useful(&mut chain) {
        error!("Unable to load useful ({e})");
    }

    if let Err(e) = load_skills(&mut chain) {
        error!("Unable to load skills ({e})");
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

    pub fn handler<C, S>(&mut self, input: S, console: &C) -> Result<()>
    where
        C: ConsoleTrait + Send + Sync,
        S: AsRef<str> + Display,
    {
        info!("input: {input}");

        if let Some(command) = input.as_ref().strip_prefix("/") {
            match command {
                "models" => self.command_models()?,
                "clear" | "reset" => command_clear()?,
                _ => {
                    let err_msg = "Command Not Found".to_string();
                    println!("{}", err_msg.red());
                    bail!("Command not found ({command})");
                }
            }
        } else {
            //
            // forward to
            //
            self.chain.link(input.as_ref(), console)?;
        }
        Ok(())
    }

    pub fn list_commands(&self) -> Vec<UserCommand> {
        vec![
            UserCommand::Help("Display help".into()),
            UserCommand::Model("Switch model".into()),
            UserCommand::Models("List available models".into()),
        ]
    }
}
