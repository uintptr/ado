use std::{
    collections::HashMap,
    env, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use base64::{Engine, prelude::BASE64_STANDARD};
use log::{error, info};
use walkdir::WalkDir;

use crate::error::{Error, Result};

use super::function_handler::get_arg;

pub struct FunctionFiles;

impl FunctionFiles {
    pub fn new() -> FunctionFiles {
        FunctionFiles {}
    }

    pub fn command_write(&self, args: &HashMap<String, String>) -> Result<String> {
        let file_name = get_arg(args, "file_name")?;
        let file_data = get_arg(args, "file_data")?;

        let file_data = BASE64_STANDARD.decode(file_data.as_bytes())?;

        let mut f = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file_name)?;

        f.write_all(&file_data)?;

        let msg = format!("{file_name} was successfully written");

        Ok(msg)
    }

    pub fn command_read(&self, args: &HashMap<String, String>) -> Result<String> {
        let file_path = get_arg(args, "file_path")?;

        let mut f = fs::OpenOptions::new().read(true).open(file_path)?;

        let mut buf = Vec::new();

        f.read_to_end(&mut buf)?;

        let encoded_data = BASE64_STANDARD.encode(buf);

        let mut msg = "data:text/plain;charset=utf-8;base64,".to_string();

        msg.push_str(&encoded_data);

        Ok(msg)
    }

    pub fn find_file<P, T>(&self, root: P, file_name: T) -> Result<PathBuf>
    where
        P: AsRef<Path>,
        T: AsRef<Path>,
    {
        let file_name = Path::new(file_name.as_ref());

        info!(
            "looking for {} in {}",
            file_name.display(),
            root.as_ref().display()
        );

        let walker = WalkDir::new(root).follow_links(false);

        for entry in walker {
            let entry = match entry {
                Ok(v) => v,
                Err(e) => {
                    error!("{e}");
                    continue;
                }
            };

            if !entry.path().is_file() {
                continue;
            }

            let cur_file_name = match entry.path().file_name() {
                Some(v) => v,
                None => continue,
            };

            if cur_file_name == file_name.as_os_str() {
                info!("found {} @ {}", file_name.display(), entry.path().display());
                return Ok(entry.path().to_path_buf());
            }
        }

        Err(Error::FileNotFoundError {
            file_path: file_name.to_path_buf(),
        })
    }

    pub fn command_find_file(&self, args: &HashMap<String, String>) -> Result<String> {
        let file_name = get_arg(args, "file_name")?;

        let cwd = env::current_dir()?;

        let file_path = self.find_file(cwd, file_name)?;

        let file_path = match file_path.to_str() {
            Some(v) => v.to_string(),
            None => {
                return Err(Error::InvalidFilePath { path: file_path });
            }
        };

        Ok(file_path)
    }
}

#[cfg(test)]
mod tests {

    use crate::staples::setup_logger;

    use super::*;

    #[test]
    fn find_file_test() {
        setup_logger(true).unwrap();

        let files = FunctionFiles::new();

        let cwd = env::current_dir().unwrap();

        info!("cwd: {}", cwd.display());

        let res = files.find_file(&cwd, "Cargo.toml");
        assert!(res.is_ok());

        let res = files.find_file(&cwd, "bleh.txt");
        assert!(res.is_err());

        if let Err(e) = res {
            error!("{e}");
        }

        let res = files.find_file(cwd, "../../../../../../etc/group");
        assert!(res.is_err());

        if let Err(e) = res {
            error!("{e}");
        }

        // Your test assertions here
    }
}
