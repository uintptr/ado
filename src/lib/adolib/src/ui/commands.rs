use crate::{
    config_file::loader::ConfigFile,
    data::types::AdoData,
    error::{Error, Result},
    llm::provider::LLMChain,
    search::google::{GoogleCSE, GoogleSearchResults},
    ui::status::StatusInfo,
};
use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};

#[derive(Parser)]
struct CommandCli {
    #[command(subcommand)]
    commands: Command,
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
    /// Google search
    #[command(alias = "s")]
    Search {
        /// query string
        #[arg(trailing_var_arg = true)]
        query: Vec<String>,
    },
    Status,
}

pub struct CommandInfo {
    pub name: String,
    pub alias: Vec<String>,
    pub about: Option<String>,
}

pub struct UserCommands {
    config: ConfigFile,
    search: GoogleCSE,
    chain: LLMChain,
}

impl UserCommands {
    pub fn new(config: &ConfigFile) -> Result<UserCommands> {
        let search = GoogleCSE::new(config)?;
        let chain = LLMChain::new(config)?;

        Ok(UserCommands {
            config: config.clone(),
            search,
            chain,
        })
    }

    pub async fn handler<S>(&mut self, line: S) -> Result<AdoData>
    where
        S: AsRef<str>,
    {
        let mut args: Vec<&str> = line.as_ref().split_whitespace().collect();

        args.insert(0, "");

        match CommandCli::try_parse_from(args) {
            Ok(c) => match c.commands {
                Command::Query { input } => {
                    let input_str = input.join(" ");

                    self.chain.query(&input_str).await
                }
                Command::Quit => Err(Error::EOF),
                Command::Reset => {
                    self.chain.reset();
                    Ok(AdoData::Reset)
                }
                Command::Search { query } => {
                    let json_str = self.search.query(query.join(" ")).await?;

                    Ok(AdoData::SearchData(GoogleSearchResults::new(json_str)))
                }
                Command::Status => {
                    let s = StatusInfo::new(&self.config, &self.chain);

                    Ok(AdoData::Status(s))
                }
            },
            Err(e) => match e.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
                    Ok(AdoData::UsageString(e.to_string()))
                }
                _ => {
                    //
                    // assuming this is query
                    //
                    self.chain.query(line.as_ref()).await
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
    use crate::{config_file::loader::ConfigFile, ui::commands::UserCommands};

    #[test]
    fn test_handler() {
        let config = ConfigFile::from_default().unwrap();

        let mut cmd = UserCommands::new(&config).unwrap();

        let _ret = cmd.handler("/help");
    }
}
