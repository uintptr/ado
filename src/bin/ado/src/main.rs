use std::{fs, path::Path};

use ado::{commands::UserCommands, console::TerminalConsole};
use adolib::{
    config::loader::AdoConfig,
    error::{Error, Result},
    llm::question::question_detection,
};
use clap::Parser;
use log::LevelFilter;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct UserArgs {
    /// Read the query from a file
    #[arg(short, long)]
    query_file: Option<String>,

    /// verbose
    #[arg(short, long)]
    verbose: bool,

    /// config file path
    #[arg(short, long)]
    config_file: Option<String>,

    /// bash command_not_found_handle
    #[arg(short, long)]
    shell_handler: Option<String>,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    query_parts: Vec<String>,
}

fn file_to_string<P>(file_path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    let data = fs::read_to_string(file_path)?;
    Ok(data)
}

async fn main_loop(mut console: TerminalConsole, command: UserCommands, opt_input: Option<String>) -> Result<()> {
    let mut init_query = opt_input;

    loop {
        let input = match init_query {
            Some(v) => v,
            None => console.read_input().await?,
        };

        //
        // process the command
        //
        let ret = command.handler(&input, &mut console).await;

        match ret {
            Ok(()) => {}
            Err(Error::Usage { help }) => console.display_string(help)?,
            Err(e @ Error::CommandNotFound { command: _ }) => console.display_error(e)?,
            Err(Error::EOF) => return Ok(()),
            Err(e) => console.display_error(e)?,
        }

        init_query = None
    }
}

fn load_config_local(local_config: &Option<String>) -> Result<AdoConfig> {
    match local_config {
        Some(v) => AdoConfig::from_path(v),
        None => AdoConfig::from_default(),
    }
}

fn init_logging(verbose: bool) {
    let level = if verbose { LevelFilter::Info } else { LevelFilter::Error };
    env_logger::builder().filter_level(level).build();
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = UserArgs::parse();

    init_logging(args.verbose);

    let query_opt = match args.shell_handler {
        Some(v) => match question_detection(&v) {
            true => Some(v),
            false => {
                println!("ado: {v}: command not found");
                return Ok(());
            }
        },
        None => match args.query_file {
            Some(v) => Some(file_to_string(v)?),
            None => match args.query_parts.is_empty() {
                true => None,
                false => Some(args.query_parts.join(" ")),
            },
        },
    };

    let _config = load_config_local(&args.config_file)?;

    let commands = UserCommands::new();

    let console = TerminalConsole::new(&commands)?;

    match main_loop(console, commands, query_opt).await {
        Ok(_) | Err(Error::EOF) => Ok(()),
        Err(e) => Err(e),
    }
}
