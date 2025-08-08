use crate::{error::Result, wasm::entry::log};

pub fn setup_wasm_logger(verbose: bool) -> Result<()> {
    let log_level = match verbose {
        true => log::LevelFilter::Info,
        false => log::LevelFilter::Warn,
    };

    fern::Dispatch::new()
        .format(|_out, message, record| {
            let now_ms = chrono::Local::now().timestamp_millis();
            let now_sec = now_ms / 1000;
            let now_ms = now_ms - (now_sec * 1000);

            let target = match record.line() {
                Some(v) => format!("{}:{v}", record.target()),
                None => record.target().to_string(),
            };

            let log_str = format!(
                "{}.{:03} :: {:<5} :: {:<45} {}",
                now_sec,
                now_ms,
                record.level(),
                target,
                message
            );

            log(&log_str);
        })
        .level(log_level)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}
