use std::{fs, path::Path};

use ado::{
    error::{Error, Result},
    llm::openai::query::OpenAI,
    staples::setup_logger,
};
use clap::Parser;
use log::info;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct UserArgs {
    /// Read the query from a file
    #[arg(short, long)]
    query_file: Option<String>,

    /// max loop
    #[arg(short, long, default_value = "10")]
    max_loop: i32,

    /// verbose
    #[arg(short, long)]
    verbose: bool,
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

fn main() -> Result<()> {
    let args = UserArgs::parse();

    setup_logger(args.verbose)?;

    let query = match args.query_file {
        Some(v) => file_to_string(v)?,
        None => {
            if args.query_parts.is_empty() {
                return Err(Error::QueryMissingError);
            }

            args.query_parts.join(" ")
        }
    };

    info!("query: {query}");

    let o = OpenAI::new()?;

    o.ask(query)
}
