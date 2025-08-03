use std::{
    env, fs,
    path::{Path, PathBuf},
};

use log::error;
use reqwest::Client;
use rstaples::staples::find_file;
use serde::Deserialize;

use crate::{
    const_vars::DOT_DIRECTORY,
    error::{Error, Result},
    storage::webdis::PersistentStorage,
};

const DEF_OPENAI_URL: &str = "https://api.openai.com/v1/responses";
const DEF_OPENAI_MODEL: &str = "gpt-4.1";

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAiConfig {
    #[serde(default = "openai_default_key")]
    pub key: String,
    #[serde(default = "openai_default_url")]
    pub url: String,
    #[serde(default = "openai_default_model")]
    pub model: String,
    pub prompt: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
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

const CONFIG_FILE_NAME: &str = "config.toml";

fn find_from_home() -> Result<PathBuf> {
    let home = env::var("HOME")?;

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

impl ConfigFile {
    pub fn from_disk() -> Result<ConfigFile> {
        let rel_config = Path::new("config").join(CONFIG_FILE_NAME);

        let config_file = match find_file(rel_config) {
            Ok(v) => v,
            Err(_) => find_from_home()?,
        };

        let file_data = fs::read_to_string(config_file)?;

        let config: ConfigFile = toml::from_str(&file_data)?;

        Ok(config)
    }

    pub fn from_string<S>(value: S) -> Result<ConfigFile>
    where
        S: AsRef<str>,
    {
        let config: ConfigFile = toml::from_str(value.as_ref())?;
        Ok(config)
    }

    pub async fn from_webdis<U, S>(user: U, server: S) -> Result<ConfigFile>
    where
        U: AsRef<str>,
        S: AsRef<str>,
    {
        let storage = PersistentStorage::new(&user, server);

        let data = storage.get_raw(user).await?;

        let config: ConfigFile = toml::from_str(&data)?;

        Ok(config)
    }

    pub async fn from_url<S>(url: S) -> Result<ConfigFile>
    where
        S: AsRef<str>,
    {
        //
        // this is a bit of a hack so we still use a cookie-less browser
        //
        let res = Client::new().get(url.as_ref()).send().await?;

        let data = match res.status().is_success() {
            true => res.text().await?,
            false => return Err(Error::HttpGetFailure),
        };

        let config: ConfigFile = toml::from_str(&data)?;

        Ok(config)
    }

    pub fn openai(&self) -> Result<&OpenAiConfig> {
        match &self.openai {
            Some(v) => Ok(v),
            None => Err(Error::ConfigNotFound),
        }
    }

    pub fn search(&self) -> Result<&GoogleConfig> {
        match &self.search {
            Some(v) => Ok(v),
            None => Err(Error::ConfigNotFound),
        }
    }
}
