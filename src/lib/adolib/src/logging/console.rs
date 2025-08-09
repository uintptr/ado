use std::sync::Once;

use rstaples::logging::StaplesLogger;

use crate::error::Result;

static INIT: Once = Once::new();

fn init_logger(verbose: bool) -> Result<()> {
    let log = StaplesLogger::new();

    let log = match verbose {
        true => log.with_stdout(),
        false => log,
    };

    log.start()?;

    Ok(())
}

pub fn setup_console_logger(verbose: bool) -> Result<()> {
    INIT.call_once(|| init_logger(verbose).unwrap());
    Ok(())
}
