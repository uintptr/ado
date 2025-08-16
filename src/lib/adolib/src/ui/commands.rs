use crate::{
    config_file::loader::ConfigFile,
    const_vars::CACHE_05_DAYS,
    data::types::AdoData,
    error::{Error, Result},
    llm::provider::LLMChain,
    search::google::{GoogleCSE, GoogleSearchResults},
    storage::{PersistentStorageTrait, persistent::PersistentStorage},
    ui::{reddit::RedditQuery, status::StatusInfo},
};
use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};

#[derive(Parser)]
struct CommandCli {
    #[command(subcommand)]
    commands: Command,
}

use log::{error, info};

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
    /// Find a sub reddit from description
    Reddit {
        /// query string
        #[arg(trailing_var_arg = true)]
        query: Vec<String>,
    },
    /// Return the first URL from the search result
    Lucky {
        /// query string
        #[arg(trailing_var_arg = true)]
        query: Vec<String>,
    },
    /// Print status information
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
    cache: PersistentStorage,
    reddit: RedditQuery,
}

impl UserCommands {
    pub fn new(config: &ConfigFile, cache: PersistentStorage) -> Result<UserCommands> {
        let search = GoogleCSE::new(config)?;
        let chain = LLMChain::new(config)?;
        let reddit = RedditQuery::new();

        Ok(UserCommands {
            config: config.clone(),
            search,
            chain,
            cache,
            reddit,
        })
    }

    async fn cached_search<S>(&self, query: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        let search_data = match self.cache.get("cmd_search", query.as_ref()).await {
            Ok(v) => {
                info!("query was cached");
                v
            }
            Err(_) => {
                let data = self.search.query(query.as_ref()).await?;

                let data = data.replace("www.reddit.com", "old.reddit.com");

                if let Err(e) = self.cache.set("cmd_search", query.as_ref(), &data, CACHE_05_DAYS).await {
                    error!("{e}");
                }

                data
            }
        };

        Ok(search_data)
    }

    async fn cached_reddit<S>(&self, query: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        let sub_reddit = match self.cache.get("cmd_reddit", query.as_ref()).await {
            Ok(v) => {
                info!("query was cached");
                v
            }
            Err(_) => {
                let data = self.reddit.find_sub(&self.chain, query.as_ref()).await?;

                if let Err(e) = self.cache.set("cmd_reddit", query.as_ref(), &data, CACHE_05_DAYS).await {
                    error!("{e}");
                }

                data
            }
        };

        Ok(sub_reddit)
    }

    async fn cached_lucky<S>(&self, query: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        let sub_reddit = match self.cache.get("cmd_lucky", query.as_ref()).await {
            Ok(v) => {
                info!("query was cached");
                v
            }
            Err(_) => {
                let data = self.search.lucky(query.as_ref()).await?;

                if let Err(e) = self.cache.set("cmd_lucky", query.as_ref(), &data, CACHE_05_DAYS).await {
                    error!("{e}");
                }

                data
            }
        };

        Ok(sub_reddit)
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
                    let query = query.join(" ");

                    let json_str = self.cached_search(query).await?;

                    Ok(AdoData::SearchData(GoogleSearchResults::new(json_str)))
                }
                Command::Reddit { query } => {
                    let query = query.join(" ");
                    let sub = self.cached_reddit(query).await?;
                    Ok(AdoData::String(sub))
                }
                Command::Lucky { query } => {
                    let query = query.join(" ");
                    let url = self.cached_lucky(query).await?;
                    Ok(AdoData::String(url))
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
    use crate::storage::persistent::PersistentStorage;
    use crate::{config_file::loader::ConfigFile, ui::commands::UserCommands};

    #[test]
    fn test_handler() {
        let config = ConfigFile::from_default().unwrap();

        let td = tempfile::Builder::new().prefix("console_test_").tempdir().unwrap();

        let cache_file = td.path().join("cache.db");

        let cache = PersistentStorage::from_path(cache_file).unwrap();

        let mut cmd = UserCommands::new(&config, cache).unwrap();

        let _ret = cmd.handler("/help");
    }
}
