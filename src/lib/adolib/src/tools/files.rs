use std::{
    env, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use base64::{Engine, prelude::BASE64_STANDARD};
use glob::glob;
use log::{error, info};
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Serialize)]
struct FileEntry {
    file_name: PathBuf,
    file_size: u64,
}

use crate::{
    data::types::AdoData,
    error::{Error, Result},
};

use super::function_args::ToolArgs;

pub struct FunctionsFiles;

impl FunctionsFiles {
    pub fn new() -> FunctionsFiles {
        FunctionsFiles {}
    }

    pub fn write(&self, args: &ToolArgs) -> Result<AdoData> {
        let file_name = args.get_string("file_name")?;
        let file_data = args.get_string("file_data")?;

        let file_data = BASE64_STANDARD.decode(file_data.as_bytes())?;

        let mut f = fs::OpenOptions::new().write(true).truncate(true).create(true).open(file_name)?;

        f.write_all(&file_data)?;

        let msg = format!("{file_name} was successfully written");

        Ok(AdoData::String(msg))
    }

    pub fn read(&self, args: &ToolArgs) -> Result<AdoData> {
        let file_path = args.get_string("file_path")?;

        let mut f = fs::OpenOptions::new().read(true).open(file_path)?;

        let mut buf = Vec::new();

        f.read_to_end(&mut buf)?;

        Ok(AdoData::Bytes(buf))
    }

    pub fn find_file<P, T>(&self, root: P, file_name: T) -> Result<PathBuf>
    where
        P: AsRef<Path>,
        T: AsRef<Path>,
    {
        let file_name = Path::new(file_name.as_ref());

        info!("looking for {} in {}", file_name.display(), root.as_ref().display());

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

    pub fn find(&self, args: &ToolArgs) -> Result<AdoData> {
        let file_name = args.get_string("file_name")?;

        let cwd = env::current_dir()?;

        let file_path = self.find_file(cwd, file_name)?;

        let file_path = match file_path.to_str() {
            Some(v) => v.to_string(),
            None => {
                return Err(Error::InvalidFilePath { path: file_path });
            }
        };

        Ok(AdoData::String(file_path))
    }

    pub fn list(&self, args: &ToolArgs) -> Result<AdoData> {
        let directory = match args.get_string("directory") {
            Ok(v) => Path::new(v).to_path_buf(),
            Err(_) => env::current_dir()?,
        };

        if !directory.exists() {
            return Err(Error::FileNotFoundError { file_path: directory });
        }

        // don't want to add recursive search just yet
        let glob_patt = format!("{}/*", directory.display());

        info!("search pattern: {glob_patt}");

        let mut files = Vec::new();

        for entry in glob(&glob_patt)? {
            let file_path = match entry {
                Ok(v) => v,
                Err(e) => {
                    error!("{e}");
                    continue;
                }
            };

            let file_name = match file_path.file_name() {
                Some(v) => Path::new(v).to_path_buf(),
                None => {
                    continue;
                }
            };

            let file_size = match fs::metadata(&file_path) {
                Ok(v) => v.len(),
                Err(e) => {
                    error!("{e}");
                    0
                }
            };

            let file_entry = FileEntry { file_name, file_size };

            info!("{}", file_path.display());

            files.push(file_entry)
        }

        if files.is_empty() {
            return Err(Error::Empty);
        }

        let ret_str = serde_json::to_string(&files)?;

        Ok(AdoData::String(ret_str))
    }
}

#[cfg(test)]
mod tests {

    use crate::logging::logger::setup_logger;

    use super::*;

    #[test]
    fn find_file_test() {
        setup_logger(true).unwrap();

        let files = FunctionsFiles::new();

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
