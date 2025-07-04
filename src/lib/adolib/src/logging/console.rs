use std::sync::Once;

use crate::error::Result;

static INIT: Once = Once::new();

fn init_logger(verbose: bool) -> Result<()> {
    let log_level = match verbose {
        true => log::LevelFilter::Info,
        false => log::LevelFilter::Warn,
    };

    fern::Dispatch::new()
        .format(|out, message, record| {
            let now_ms = chrono::Local::now().timestamp_millis();
            let now_sec = now_ms / 1000;
            let now_ms = now_ms - (now_sec * 1000);

            let target = match record.line() {
                Some(v) => format!("{}:{v}", record.target()),
                None => record.target().to_string(),
            };

            out.finish(format_args!(
                "{}.{:03} :: {:<5} :: {:<45} {}",
                now_sec,
                now_ms,
                record.level(),
                target,
                message
            ))
        })
        .level(log_level)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

pub fn setup_console_logger(verbose: bool) -> Result<()> {
    INIT.call_once(|| init_logger(verbose).unwrap());
    Ok(())
}
