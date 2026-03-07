use std::fs;

use ado::{commands::UserCommands, console::TerminalConsole, spinner::AdoSpinner};
use adolib::config::loader::AdoConfig;
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

fn main_loop(mut console: TerminalConsole, mut command: UserCommands) -> Result<()> {
    let mut spinner = AdoSpinner::new();

    loop {
        let input = match console.read_input() {
            Ok(v) => v,
            Err(e) => {
                if matches!(
                    e.downcast_ref::<adolib::error::Error>(),
                    Some(adolib::error::Error::EOF)
                ) {
                    break;
                }
                return Err(e.into());
            }
        };

        spinner.start();

        //
        // process the command
        //
        if let Err(e) = command.handler(&input, |data| {
            spinner.stop();

            if let Err(e) = console.display_data(data) {
                error!("Unable to display data ({e})");
            }
            Ok(())
        }) {
            spinner.stop();
            error!("{e}");
        }
    }

    spinner.quit()?;

    Ok(())
}

fn load_config_local(local_config: &Option<String>) -> Result<AdoConfig> {
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
        .write(true)
        .create(true)
        .open(&log_file)
        .with_context(|| format!("Unable to open {} for writing", log_file.display()))?;

    println!("log file: {}", log_file.display());

    let target = env_logger::Target::Pipe(Box::new(log_fd));

    let level = if verbose { LevelFilter::Info } else { LevelFilter::Error };
    env_logger::builder().filter_level(level).target(target).init();
    Ok(())
}

fn main() -> Result<()> {
    let args = UserArgs::parse();

    init_logging(args.verbose)?;

    let config = load_config_local(&args.config_file)?;

    let commands = UserCommands::new(&config)?;

    let console = TerminalConsole::new(&commands)?;

    main_loop(console, commands)
}
