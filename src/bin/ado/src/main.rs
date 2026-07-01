use std::fs;

use ado::{commands::UserCommands, headless::headless_run};
use adolib::{cache::kv::KVCache, config::loader::AdoConfig, error::Error};
use anyhow::{Context, Result};
use clap::Parser;
use log::LevelFilter;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct UserArgs {
    /// verbose
    #[arg(short, long)]
    verbose: bool,

    /// headless communicates thru stdin / stdout
    #[arg(long)]
    headless: bool,

    /// config file path
    #[arg(short, long)]
    config_file: Option<String>,
}

fn load_config_local(local_config: Option<&String>) -> Result<AdoConfig> {
    match local_config {
        Some(v) => AdoConfig::from_path(v),
        None => AdoConfig::from_default(),
    }
    .context("Unable to load local config")
}

fn init_logging(verbose: bool) {
    let level = if verbose {
        LevelFilter::Info
    } else {
        LevelFilter::Error
    };
    env_logger::builder().filter_level(level).init();
}

fn main() -> Result<()> {
    let args = UserArgs::parse();

    init_logging(true);

    let config = load_config_local(args.config_file.as_ref())?;
    let cache = KVCache::default_path().context("Unable to initialize kv cache")?;

    let commands = UserCommands::new(&config, &cache)?;

    if args.headless {
        // Headless has no config dir dependency: config comes from
        // --config-file and the cache from ADO_CACHE_DIRECTORY. Avoid touching
        // dirs::config_dir() so it works when running as a bare uid with no
        // $HOME (e.g. an unprivileged container user).
        headless_run(commands)
    } else {
        let config_dir = dirs::config_dir().ok_or(Error::ConfigNotFound)?;
        let history_file = config_dir.join("ado").join("history.txt");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        ado::tui_app::run(commands, &history_file)
    }
}
