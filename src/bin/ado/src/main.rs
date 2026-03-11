use std::fs;

use ado::commands::UserCommands;
use adolib::{config::loader::AdoConfig, error::Error};
use anyhow::{Context, Result, anyhow};
use clap::Parser;
use log::{LevelFilter, error};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct UserArgs {
    /// verbose
    #[arg(short, long)]
    verbose: bool,

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

fn init_logging(verbose: bool) -> Result<()> {
    let data_dir = dirs::data_dir().ok_or_else(|| anyhow!("Unable to find data dir"))?;

    let pkg_name = env!("CARGO_PKG_NAME");

    let log_dir = data_dir.join(pkg_name);

    if !log_dir.exists() {
        fs::create_dir_all(&log_dir).with_context(|| format!("Unable to create {}", log_dir.display()))?;
    }

    let log_file = log_dir.join(format!("{pkg_name}.log"));

    let log_fd = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&log_file)
        .with_context(|| format!("Unable to open {} for writing", log_file.display()))?;

    let target = env_logger::Target::Pipe(Box::new(log_fd));

    let level = if verbose { LevelFilter::Info } else { LevelFilter::Error };
    env_logger::builder().filter_level(level).target(target).init();
    Ok(())
}

fn main() -> Result<()> {
    let args = UserArgs::parse();

    init_logging(true)?;

    let config = load_config_local(args.config_file.as_ref())?;

    let commands = UserCommands::new(&config)?;

    // Build the list of command names and history path before moving commands
    let command_names: Vec<String> = commands.list_commands().iter().map(|c| c.name().to_string()).collect();

    let config_dir = dirs::config_dir().ok_or(Error::ConfigNotFound)?;
    let history_file = config_dir.join("history.txt");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    let history = ado::tui_app::load_history(&history_file);

    if let Err(e) = ado::tui_app::run(commands, history, &history_file, command_names) {
        error!("{e}");
    }

    Ok(())
}
