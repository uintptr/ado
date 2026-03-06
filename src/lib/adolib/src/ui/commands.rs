use crate::{
    config::loader::AdoConfig,
    data::types::AdoData,
    error::{Error, Result},
    llm::chain::LLMChain,
    ui::{ConsoleDisplayTrait, status::StatusInfo},
};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum, error::ErrorKind};

#[derive(Parser)]
struct CommandCli {
    #[command(subcommand)]
    commands: Command,
}

use log::error;

#[derive(ValueEnum, Clone, Debug)]
enum McpState {
    Enable,
    Disable,
}

#[derive(Debug, Subcommand)]
enum LlmCommmands {
    /// Provider
    Provider { llm: Option<String> },
    /// LLM usage
    Usage,
    /// Chain
    Chain,
    /// Model
    Model { model: Option<String> },
}

#[derive(Debug, Subcommand)]
enum McpCommmands {
    /// Load
    Load { name: String },
    /// List
    List,
    /// List Tools
    Tools,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// query the LLM
    #[command(alias = "q")]
    Query {
        /// query string
        #[arg(trailing_var_arg = true)]
        input: Vec<String>,
    },
    /// reset the input context
    #[command(alias = "r")]
    Reset,
    /// quit
    #[command(alias = "exit")]
    Quit,
    /// Print status information
    Status,
    /// LLM related Commands
    Llm {
        #[command(subcommand)]
        command: LlmCommmands,
    },
}

pub struct CommandInfo {
    pub name: String,
    pub alias: Vec<String>,
    pub about: Option<String>,
}

pub struct UserCommands {
    config: AdoConfig,
    chain: LLMChain,
}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////

impl UserCommands {
    pub fn new(config: &AdoConfig) -> Result<UserCommands> {
        let chain = LLMChain::new(config)?;

        Ok(UserCommands {
            config: config.clone(),
            chain,
        })
    }

    async fn update_llm<S>(&mut self, llm: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        let mut new_config = self.config.clone();

        new_config.update_llm(llm);

        let new_chain = LLMChain::new(&new_config)?;

        self.config = new_config;
        self.chain = new_chain;

        if let Err(e) = self.config.sync().await {
            error!("{e}");
        }

        Ok(())
    }

    async fn command_table<S, C>(&mut self, line: S, console: &mut C) -> Result<()>
    where
        S: AsRef<str>,
        C: ConsoleDisplayTrait,
    {
        let mut args: Vec<&str> = line.as_ref().split_whitespace().collect();

        console.start_spinner();

        args.insert(0, "");

        match CommandCli::try_parse_from(args) {
            Ok(c) => match c.commands {
                Command::Query { input } => {
                    let input_str = input.join(" ");

                    self.chain.link(&input_str, console).await
                }
                Command::Quit => Err(Error::EOF),
                Command::Reset => {
                    self.chain.reset();
                    console.display(AdoData::Reset)
                }
                Command::Status => {
                    let s = StatusInfo::new(&self.config, &self.chain);
                    console.display(AdoData::Status(s))
                }
                Command::Llm { command } => match command {
                    LlmCommmands::Provider { llm } => {
                        if let Some(llm) = llm {
                            let cur_llm = self.config.llm_provider();

                            if cur_llm != llm {
                                self.update_llm(llm).await?;
                            }
                        }

                        let llm = self.config.llm_provider();
                        // TODO XXX TODO
                        // can use use a &str here
                        console.display_string(llm) // can we use a str here
                    }
                    LlmCommmands::Usage => {
                        let usage = self.chain.usage();
                        console.display(AdoData::LlmUsage(usage))
                    }
                    LlmCommmands::Model { model } => {
                        if let Some(model) = model {
                            let cur_model = self.chain.model();

                            if cur_model != model {
                                self.chain.change_model(model);
                            }
                        }

                        let model = self.chain.model();
                        console.display_string(model)
                    }
                    LlmCommmands::Chain => {
                        let data = self.chain.dump_chain()?;
                        console.display(data)
                    }
                },
            },
            Err(e) => match e.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
                    console.display(AdoData::UsageString(e.to_string()))
                }
                ErrorKind::InvalidValue => {
                    let err_msg = format!("{e}");
                    console.display_string(err_msg)
                }
                ErrorKind::MissingRequiredArgument => {
                    let err_msg = format!("{e}");
                    console.display_string(err_msg)
                }
                _ => {
                    //error!("{e}");
                    //
                    // assume it's a query
                    //
                    self.chain.link(line.as_ref(), console).await
                }
            },
        }
    }

    pub async fn handler<S, C>(&mut self, line: S, console: &mut C) -> Result<()>
    where
        S: AsRef<str>,
        C: ConsoleDisplayTrait,
    {
        console.start_spinner();

        let ret = self.command_table(line, console).await;

        console.stop_spinner();

        ret
    }

    pub fn list_commands(&self) -> Vec<CommandInfo> {
        let mut commands = Vec::new();

        for s in CommandCli::command().get_subcommands() {
            let mut alias = Vec::new();

            for a in s.get_aliases() {
                alias.push(a.to_string());
            }

            let about = s.get_about().map(|v| v.to_string());

            let info = CommandInfo {
                name: s.get_name().to_string(),
                alias,
                about,
            };

            commands.push(info)
        }

        commands
    }
}

///////////////////////////////////////////////////////////////////////////////
// TESTS
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::config::loader::AdoConfig;
    use crate::ui::NopConsole;
    use crate::ui::commands::UserCommands;

    #[test]
    fn test_handler() {
        let config = AdoConfig::from_default().unwrap();

        let mut cmd = UserCommands::new(&config).unwrap();

        let mut console = NopConsole::new();

        let _ret = cmd.handler("/help", &mut console);
    }
}
