use std::{
    fs,
    path::{Path, PathBuf},
};

use rstaples::staples::find_file;

use crate::{
    const_vars::DOT_DIRECTORY,
    error::{Error, Result},
};

use super::file::ConfigFile;

const CONFIG_FILE_NAME: &str = "config.toml";

pub fn find_from_home() -> Result<PathBuf> {
    let home = home::home_dir().ok_or(Error::HomeDirNotFound)?;

    let dot_dir = Path::new(&home).join(DOT_DIRECTORY);

    if !dot_dir.exists() {
        return Err(Error::FileNotFoundError { file_path: dot_dir });
    }

    let config_file = dot_dir.join(CONFIG_FILE_NAME);

    match config_file.exists() {
        true => Ok(config_file),
        false => Err(Error::FileNotFoundError { file_path: config_file }),
    }
}

pub fn from_file() -> Result<ConfigFile> {
    let rel_config = Path::new("config").join(CONFIG_FILE_NAME);

    let config_file = match find_file(rel_config) {
        Ok(v) => v,
        Err(_) => find_from_home()?,
    };

    let file_data = fs::read_to_string(config_file)?;

    let config: ConfigFile = toml::from_str(&file_data)?;

    Ok(config)
}
