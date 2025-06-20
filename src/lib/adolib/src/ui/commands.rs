use crate::{
    config::file::ConfigFile,
    data::AdoData,
    error::{Error, Result},
    llm::openai::chain::AIChain,
    search::google::GoogleCSE,
};
use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};

#[derive(Parser)]
struct CommandCli {
    #[command(subcommand)]
    commands: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
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
    Quit,
    /// Google search
    #[command(alias = "s")]
    Search {
        /// query string
        #[arg(trailing_var_arg = true)]
        query: Vec<String>,
    },
}

pub struct CommandInfo {
    pub name: String,
    pub alias: Vec<String>,
    pub about: Option<String>,
}

pub struct UserCommands {
    search: GoogleCSE,
    chain: AIChain,
}

#[derive(Debug)]
pub struct CommandResponse {
    pub command: Command,
    pub data: Option<AdoData>,
}

impl UserCommands {
    pub fn new(config: &ConfigFile) -> Result<UserCommands> {
        let search = GoogleCSE::new(config)?;
        let chain = AIChain::new(config)?;

        Ok(UserCommands { search, chain })
    }

    pub async fn handler<S>(&mut self, line: S) -> Result<CommandResponse>
    where
        S: AsRef<str>,
    {
        let mut args = shell_words::split(line.as_ref())?;

        args.insert(0, "".to_string());

        match CommandCli::try_parse_from(args) {
            Ok(c) => match c.commands {
                Command::Query { input } => {
                    let input_str = input.join(" ");

                    let rep = self.chain.query(input_str).await?;

                    Ok(CommandResponse {
                        command: Command::Query { input },
                        data: Some(rep),
                    })
                }
                Command::Quit => Err(Error::EOF),
                Command::Reset => {
                    self.chain.reset();
                    Ok(CommandResponse {
                        command: Command::Reset,
                        data: None,
                    })
                }
                Command::Search { query } => {
                    let json_str = self.search.query(query.join(" ")).await?;

                    Ok(CommandResponse {
                        command: Command::Search { query },
                        data: Some(AdoData::Json(json_str)),
                    })
                }
            },
            Err(e) => match e.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
                    let usage = e.to_string();
                    Err(Error::Usage { help: usage })
                }
                _ => {
                    //
                    // assuming this is query
                    //
                    let rep = self.chain.query(&line).await?;

                    Ok(CommandResponse {
                        command: Command::Query {
                            input: vec![line.as_ref().to_string()],
                        },
                        data: Some(rep),
                    })
                }
            },
        }
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

#[cfg(test)]
mod tests {
    use crate::{config::file::ConfigFile, ui::commands::UserCommands};

    #[test]
    fn test_handler() {
        let config = ConfigFile::load().unwrap();

        let mut cmd = UserCommands::new(&config).unwrap();

        let _ret = cmd.handler("/help");
    }
}
