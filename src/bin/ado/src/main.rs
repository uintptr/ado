use ado::{commands::UserCommands, console::TerminalConsole};
use adolib::config::loader::AdoConfig;
use anyhow::{Context, Result};
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

        //
        // process the command
        //
        if let Err(e) = command.handler(&input, |data| {
            if let Err(e) = console.display_data(data) {
                error!("Unable to display data ({e})");
            }
            Ok(())
        }) {
            error!("{e}");
        }
    }

    Ok(())
}

fn load_config_local(local_config: &Option<String>) -> Result<AdoConfig> {
    match local_config {
        Some(v) => AdoConfig::from_path(v),
        None => AdoConfig::from_default(),
    }
    .context("Unable to load local config")
}

fn init_logging(verbose: bool) {
    let level = if verbose { LevelFilter::Info } else { LevelFilter::Error };
    env_logger::builder().filter_level(level).init()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = UserArgs::parse();

    init_logging(args.verbose);

    let config = load_config_local(&args.config_file)?;

    let commands = UserCommands::new(&config)?;

    let console = TerminalConsole::new(&commands)?;

    main_loop(console, commands)
}
