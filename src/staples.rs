use std::{
    env,
    path::{Path, PathBuf},
};

use crate::error::{Error, Result};

const HOME_CONFIG_DIR: &str = ".ado";

fn find_file_cwd<P>(file_name: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    let cwd = env::current_dir()?;

    let file_path = cwd.join(file_name);

    match file_path.exists() {
        true => Ok(file_path),
        false => Err(Error::FileNotFoundError { file_path }),
    }
}

fn find_file_sxs<P>(file_name: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    let prog_dir = env::current_exe()?;

    let dirname = prog_dir.parent().ok_or(Error::DirnameError)?;

    let file_path = dirname.join(file_name);

    match file_path.exists() {
        true => Ok(file_path),
        false => Err(Error::FileNotFoundError { file_path }),
    }
}

fn find_file_home<P>(file_name: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    let home = env::var("HOME")?;

    let file_path = Path::new(&home).join(HOME_CONFIG_DIR).join(file_name);

    match file_path.exists() {
        true => Ok(file_path),
        false => Err(Error::FileNotFoundError { file_path }),
    }
}

pub fn find_file<P>(file_name: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    match find_file_sxs(&file_name) {
        Ok(v) => Ok(v),
        Err(_) => match find_file_cwd(&file_name) {
            Ok(v) => Ok(v),
            Err(_) => find_file_home(file_name),
        },
    }
}

pub fn setup_logger(verbose: bool) -> Result<()> {
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
                "{}.{:03} :: {:<5} :: {:<35} {}",
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
