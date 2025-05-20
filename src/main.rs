use ado::{
    error::{Error, Result},
    genini::Gemini,
    logging::setup_logger,
};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct UserArgs {
    /// Use this url instead of the one in the config file
    #[arg(short, long)]
    url: Option<String>,
    /// verbose
    #[arg(short, long)]
    verbose: bool,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    query_parts: Vec<String>,
}

fn main() -> Result<()> {
    let args = UserArgs::parse();

    setup_logger(args.verbose)?;

    if args.query_parts.is_empty() {
        return Err(Error::QueryMissingError);
    }

    let query = args.query_parts.join(" ");

    let mut g = Gemini::new()?;

    if let Some(url) = args.url {
        g.with_url(url);
    }

    let resp = g.ask(query)?;

    println!("{resp}");

    Ok(())
}
