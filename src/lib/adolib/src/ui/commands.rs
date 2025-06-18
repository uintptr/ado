use log::error;

use crate::{
    config::file::ConfigFile,
    data::AdoData,
    error::{Error, Result},
    search::google::GoogleCSE,
};
use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};

#[derive(Parser)]
struct Command {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// reset the input context
    Reset,
    /// quit
    Quit,
    /// Google search
    Search {
        /// query string
        #[arg(trailing_var_arg = true)]
        query: Vec<String>,
    },
}

pub struct UserCommands {
    search: GoogleCSE,
}

impl UserCommands {
    pub fn new(config: &ConfigFile) -> Result<UserCommands> {
        let search = GoogleCSE::new(config)?;

        Ok(UserCommands { search })
    }

    pub async fn handler(&self, line: &str) -> Result<AdoData> {
        let line: String = line.chars().skip(1).collect();
        let mut args = shell_words::split(&line)?;

        args.insert(0, "".to_string());

        let res = match Command::try_parse_from(args) {
            Ok(c) => match c.commands {
                Commands::Quit => return Err(Error::EOF),
                Commands::Reset => return Err(Error::ResetInput),
                Commands::Search { query } => {
                    let json_str = self.search.query(query.join(" ")).await?;
                    AdoData::Json(json_str)
                }
            },
            Err(e) => match e.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
                    AdoData::String(e.to_string())
                }
                _ => {
                    error!("{e}");
                    return Err(Error::ConfigNotFound);
                }
            },
        };

        Ok(res)
    }

    pub fn usage(&self) -> String {
        Command::command().render_long_version()
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::file::ConfigFile, ui::commands::UserCommands};

    #[test]
    fn test_handler() {
        let config = ConfigFile::load().unwrap();

        let cmd = UserCommands::new(&config).unwrap();

        let _ret = cmd.handler("/help");
    }
}
