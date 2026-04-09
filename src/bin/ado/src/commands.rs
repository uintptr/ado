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

use crate::intrinsics::IntrinsicPrompts;

pub struct UserCommands {
    chain: LLMChain,
    commands: Vec<Box<dyn UserCommansTrait + 'static>>,
}

pub trait UserCommansTrait: Send {
    fn name(&self) -> &'static str;
    fn callback(&mut self, input: &str, chain: &mut LLMChain, console: &dyn ConsoleTrait);
}

struct CommandHelp;
struct CommandModels;
struct CommandReset;
struct CommandModel;

impl UserCommansTrait for CommandModel {
    fn name(&self) -> &'static str {
        "model"
    }

    fn callback(&mut self, input: &str, chain: &mut LLMChain, console: &dyn ConsoleTrait) {
        let input = input.trim_ascii_start().trim_ascii_end();

        if !input.is_empty() {
            //
            // Check that the model actually exists
            //
            let models = chain.models();

            for m in &models {
                info!("model: {m} != {input}");
            }

            let exists = models.contains(&input.to_string());

            if !exists {
                let err_msg = format!("{input} does not exist. See /models");
                console.error_message(&err_msg);
                return;
            }

            if let Err(e) = chain.change_model(input) {
                error!("Unable to change model. {e}");
            }
        }

        let s = format!("# Model: {}", chain.model());
        console.print_markdown(&s);
    }
}

impl UserCommansTrait for CommandReset {
    fn name(&self) -> &'static str {
        "reset"
    }

    fn callback(&mut self, _input: &str, _chain: &mut LLMChain, _console: &dyn ConsoleTrait) {
        let mut stdout = io::stdout();
        print!("{esc}c", esc = 27 as char);

        if let Err(e) = stdout.flush() {
            error!("{e}");
        }
    }
}

impl UserCommansTrait for CommandHelp {
    fn name(&self) -> &'static str {
        "help"
    }

    fn callback(&mut self, _input: &str, _chain: &mut LLMChain, console: &dyn ConsoleTrait) {
        console.print_markdown(
            "# Commands\n\
             * `/help` — show this help\n\
             * `/model [name]` — show or switch the current model\n\
             * `/models` — list all available models\n\
             * `/reset` — clear the terminal screen\n\
             \n\
             ## Completion\n\
             * Type `/` to trigger command completion\n\
             * Type `@path` to attach a file (Tab to browse)",
        );
    }
}

impl UserCommansTrait for CommandModels {
    fn name(&self) -> &'static str {
        "models"
    }

    fn callback(&mut self, _input: &str, chain: &mut LLMChain, console: &dyn ConsoleTrait) {
        let mut output = Vec::new();

        let cur_model = chain.model();

        output.push("Models:".to_string());

        for m in chain.models() {
            if m == cur_model {
                output.push(format!("* **{m}**"));
            } else {
                output.push(format!("* {m}"));
            }
        }

        console.print_markdown(&output.join("\n"));
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

fn load_useful(chain: &mut LLMChain) {
    if let Ok(cwd) = env::current_dir() {
        let cwd_prompt = format!("The current working directory is {}", cwd.display());
        chain.add_content(LLMRole::User, cwd_prompt);
    }

    let current_os = format!("The current operating system is {OS}");
    chain.add_content(LLMRole::User, current_os);
}

fn load_skills_from_path(chain: &mut LLMChain, path: &Path) -> Result<()> {
    info!("Loading skills from {}", path.display());

    let glob_pattern = format!("{}/*.md", path.display());

    for md_file in glob::glob(&glob_pattern)?.flatten() {
        if let Ok(data) = fs::read_to_string(md_file) {
            chain.add_content(LLMRole::System, data);
        }
    }

    Ok(())
}

fn load_skills(chain: &mut LLMChain) {
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
}

fn init_chain(config: &AdoConfig) -> Result<LLMChain> {
    let mut chain = LLMChain::new(config)?;

    load_intrinsics(&mut chain);

    if let Err(e) = load_ado_md(&mut chain) {
        error!("Unable to load ADO.md files ({e})");
    }

    load_useful(&mut chain);

    load_skills(&mut chain);

    Ok(chain)
}

impl UserCommands {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        let chain = init_chain(config).context("Unable to initialize llm chain")?;

        let commands: Vec<Box<dyn UserCommansTrait>> = vec![
            Box::new(CommandHelp {}),
            Box::new(CommandModels {}),
            Box::new(CommandReset {}),
            Box::new(CommandModel {}),
        ];

        Ok(Self { chain, commands })
    }

    pub fn current_model(&self) -> String {
        self.chain.model().to_string()
    }

    pub fn command_models<C>(&self, console: &C) -> Result<()>
    where
        C: ConsoleTrait + Send + Sync,
    {
        let mut output = Vec::new();

        let cur_model = self.chain.model();

        output.push("Models:".to_string());

        for m in self.chain.models() {
            if m == cur_model {
                output.push(format!("* **{m}**"));
            } else {
                output.push(format!("* {m}"));
            }
        }

        console.print_markdown(&output.join("\n"));

        Ok(())
    }

    pub fn handler<C, S>(&mut self, input: S, console: &C) -> Result<()>
    where
        C: ConsoleTrait + Send + Sync,
        S: AsRef<str> + Display,
    {
        info!("input: {input}");

        if let Some(command) = input.as_ref().strip_prefix("/") {
            for c in &mut self.commands {
                if let Some(args) = command.strip_prefix(c.name()) {
                    c.callback(args, &mut self.chain, console);
                    return Ok(());
                }
            }

            console.print_markdown("**Command Not Found**");
            bail!("Command not found ({command})");
        }
        //
        // forward to
        //
        self.chain.link(input.as_ref(), console)?;
        Ok(())
    }

    #[must_use]
    pub fn list_commands(&self) -> &[Box<dyn UserCommansTrait>] {
        &self.commands
    }
}
