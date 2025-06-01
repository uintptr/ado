use std::{fs, path::Path};

use ado::shell::detect_shell_question;
use adolib::{
    config::file::ConfigFile,
    error::{Error, Result},
    llm::openai::query::OpenAI,
    staples::setup_logger,
    ui::ux::Console,
};
use clap::Parser;

use log::error;

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

    let query_opt = match args.shell_handler {
        Some(v) => match detect_shell_question(&v) {
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

    let mut console = Console::new()?;

    let mut o = OpenAI::new(&config)?;

    if let Some(query) = query_opt {
        o.with_initial_query(query);
    }

    match o.ask(&mut console).await {
        Ok(()) => Ok(()),
        // CTRL+C or CTRL+D are ok, we still want to return success
        Err(Error::EOF) => Ok(()),
        Err(e) => {
            error!("{e}");
            Err(e)
        }
    }
}
