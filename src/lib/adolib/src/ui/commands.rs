use crate::{
    config::loader::AdoConfig,
    const_vars::CACHE_05_DAYS,
    data::types::AdoData,
    error::{Error, Result},
    llm::chain::LLMChain,
    search::google::{GoogleCSE, GoogleSearchResults},
    storage::{PersistentStorageTrait, persistent::PersistentStorage},
    ui::{ConsoleDisplayTrait, reddit::RedditQuery, status::StatusInfo},
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
    /// LLM provider
    Llm { llm: Option<String> },
    /// LLM usage
    Usage,
    /// Model
    Model { model: Option<String> },
}

pub struct CommandInfo {
    pub name: String,
    pub alias: Vec<String>,
    pub about: Option<String>,
}

pub struct UserCommands {
    config: AdoConfig,
    search: GoogleCSE,
    chain: LLMChain,
    cache: PersistentStorage,
    reddit: RedditQuery,
}

impl UserCommands {
    pub fn new(config: &AdoConfig, cache: PersistentStorage) -> Result<UserCommands> {
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

                let data = data.replace("www.reddit.com", "old.reddit.com");

                if let Err(e) = self.cache.set("cmd_lucky", query.as_ref(), &data, CACHE_05_DAYS).await {
                    error!("{e}");
                }

                data
            }
        };

        Ok(sub_reddit)
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
                Command::Search { query } => {
                    let query = query.join(" ");

                    let json_str = self.cached_search(query).await?;

                    let data = AdoData::SearchData(GoogleSearchResults::new(json_str));

                    console.display(data)
                }
                Command::Reddit { query } => {
                    let query = query.join(" ");
                    let sub = self.cached_reddit(query).await?;
                    console.display_string(sub)
                }
                Command::Lucky { query } => {
                    let query = query.join(" ");
                    let url = self.cached_lucky(query).await?;
                    console.display_string(url)
                }
                Command::Status => {
                    let s = StatusInfo::new(&self.config, &self.chain);
                    console.display(AdoData::Status(s))
                }
                Command::Llm { llm } => {
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
                Command::Usage => {
                    let usage = self.chain.usage();
                    console.display(AdoData::LlmUsage(usage))
                }
                Command::Model { model } => {
                    if let Some(model) = model {
                        let cur_model = self.chain.model();

                        if cur_model != model {
                            self.chain.change_model(model);
                        }
                    }

                    let model = self.chain.model();
                    console.display_string(model)
                }
            },
            Err(e) => match e.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
                    console.display(AdoData::UsageString(e.to_string()))
                }
                _ => {
                    //
                    // assuming this is query
                    //
                    self.chain.link(line.as_ref(), console).await?;
                    Ok(())
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

#[cfg(test)]
mod tests {
    use crate::config::loader::AdoConfig;
    use crate::storage::persistent::PersistentStorage;
    use crate::ui::NopConsole;
    use crate::ui::commands::UserCommands;

    #[test]
    fn test_handler() {
        let config = AdoConfig::from_default().unwrap();

        let td = tempfile::Builder::new().prefix("console_test_").tempdir().unwrap();

        let cache_file = td.path().join("cache.db");

        let cache = PersistentStorage::from_path(cache_file).unwrap();

        let mut cmd = UserCommands::new(&config, cache).unwrap();

        let mut console = NopConsole {};

        let _ret = cmd.handler("/help", &mut console);
    }
}
