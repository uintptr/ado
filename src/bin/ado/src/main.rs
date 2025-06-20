use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use ado::console::ConsoleUI;
use adolib::{
    config::file::ConfigFile,
    error::{Error, Result},
    llm::question::question_detection,
    logging::logger::setup_logger,
    ui::commands::UserCommands,
};
use clap::Parser;
use log::info;
use spinner::SpinnerBuilder;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct UserArgs {
    /// Read the query from a file
    #[arg(short, long)]
    query_file: Option<String>,

    /// verbose
    #[arg(short, long)]
    verbose: bool,

    /// remote config url ( mainly for the wasm bits )
    #[arg(short, long, default_value = "http://10.0.0.2/ado.toml")]
    remote_config_url: Option<String>,

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

async fn main_loop(mut console: ConsoleUI, mut command: UserCommands, opt_input: Option<String>) -> Result<()> {
    let mut init_query = opt_input;

    loop {
        let input = match init_query {
            Some(v) => v,
            None => console.read_input().await?,
        };

        //
        // little spinner waiting for the response
        //
        let spinner = SpinnerBuilder::new("".into()).start();

        //
        // process the command
        //
        let ret = command.handler(&input).await;

        spinner.close();
        print!(" \r ");
        io::stdout().flush()?;

        info!("{ret:?}");

        match ret {
            Ok(v) => console.display_messages(&v)?,
            Err(Error::Usage { help }) => console.display_string(help)?,
            Err(e @ Error::CommandNotFound { command: _ }) => console.display_error(e)?,
            Err(Error::EOF) => return Ok(()),
            Err(e) => return Err(e),
        }

        init_query = None
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = UserArgs::parse();

    setup_logger(args.verbose)?;

    let query_opt = match args.shell_handler {
        Some(v) => match question_detection(&v) {
            true => Some(v),
            false => {
                println!("ado: {}: command not found", v);
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

    let config = match ConfigFile::load() {
        Ok(v) => v,
        Err(e) => match args.remote_config_url {
            Some(v) => ConfigFile::load_with_url(&v).await?,
            None => return Err(e),
        },
    };

    let console = ConsoleUI::new(&config)?;

    let command = UserCommands::new(&config)?;

    match main_loop(console, command, query_opt).await {
        Ok(_) | Err(Error::EOF) => Ok(()),
        Err(e) => Err(e),
    }
}
