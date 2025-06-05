use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use ado::console::ConsoleUI;
use adolib::{
    config::file::ConfigFile,
    error::{Error, Result},
    llm::{openai::chain::AIChain, question::question_detection},
    staples::setup_logger,
};
use clap::Parser;
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
#[tokio::main]

async fn main() -> Result<()> {
    let args = UserArgs::parse();

    setup_logger(args.verbose)?;

    let mut query_opt = match args.shell_handler {
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

    let mut console = ConsoleUI::new()?;

    let mut chain = AIChain::new(&config)?;

    loop {
        let query = match query_opt {
            Some(v) => v,
            None => match console.read_input() {
                Ok(v) => v,
                Err(Error::ResetInput) => {
                    chain.reset();
                    continue;
                }
                Err(Error::EOF) => break Ok(()),
                Err(e) => break Err(e),
            },
        };

        //
        // little spinner waiting for the response
        //
        let spinner = SpinnerBuilder::new("".into()).start();

        //
        // query the LLM
        //
        let msgs = chain.query(query).await?;

        spinner.close();
        print!("\r ");
        io::stdout().flush()?;

        console.display_messages(&msgs)?;

        query_opt = None
    }
}
