use std::{
    env, fs,
    path::{Path, PathBuf},
};

use log::error;
use serde::Deserialize;

use crate::{
    const_vars::DOT_DIRECTORY,
    error::{Error, Result},
    staples::find_file,
};

const CONFIG_FILE_NAME: &str = "config.toml";
const DEF_OPENAI_URL: &str = "https://api.openai.com/v1/responses";
const DEF_OPENAI_MODEL: &str = "gpt-4.1";

#[derive(Debug, Deserialize)]
pub struct OpenAiConfig {
    #[serde(default = "openai_default_key")]
    pub key: String,
    #[serde(default = "openai_default_url")]
    pub url: String,
    #[serde(default = "openai_default_model")]
    pub model: String,
    pub prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleConfig {
    pub cx: String,
    pub geo: String,
    pub key: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    openai: Option<OpenAiConfig>,
    search: Option<GoogleConfig>,
}

fn openai_default_url() -> String {
    DEF_OPENAI_URL.to_string()
}

fn openai_default_model() -> String {
    DEF_OPENAI_MODEL.to_string()
}

fn openai_default_key() -> String {
    match env::var("OPENAI_API_KEY") {
        Ok(v) => v,
        Err(_) => {
            error!("OPENAI_API_KEY env variable not defined");
            "".to_string()
        }
    }
}

fn find_from_home() -> Result<PathBuf> {
    let home = env::home_dir().ok_or(Error::HomeDirNotFound)?;

    let dot_dir = Path::new(&home).join(DOT_DIRECTORY);

    if !dot_dir.exists() {
        return Err(Error::FileNotFoundError { file_path: dot_dir });
    }

    let config_file = dot_dir.join(CONFIG_FILE_NAME);

    match config_file.exists() {
        true => Ok(config_file),
        false => Err(Error::FileNotFoundError {
            file_path: config_file,
        }),
    }
}

fn from_file() -> Result<ConfigFile> {
    let rel_config = Path::new("config").join(CONFIG_FILE_NAME);

    let config_file = match find_file(rel_config) {
        Ok(v) => v,
        Err(_) => find_from_home()?,
    };

    let file_data = fs::read_to_string(config_file)?;

    let config: ConfigFile = toml::from_str(&file_data)?;

    Ok(config)
}

impl ConfigFile {
    pub fn load() -> Result<ConfigFile> {
        let config = match from_file() {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        Ok(config)
    }

    pub fn load_with_url(url: String) -> Result<ConfigFile> {
        //
        // this is a bit of a hack so we still use a cookie-less browser
        //
        let res = minreq::get(url).send()?;

        let data = match res.status_code {
            200..299 => res.as_str()?,
            _ => return Err(Error::HttpGetFailure),
        };

        let config: ConfigFile = toml::from_str(data)?;

        Ok(config)
    }

    pub fn openai(self) -> Result<OpenAiConfig> {
        self.openai.ok_or(Error::ConfigNotFound)
    }

    pub fn search(self) -> Result<GoogleConfig> {
        self.search.ok_or(Error::ConfigNotFound)
    }
}
