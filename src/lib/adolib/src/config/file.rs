use std::env;

use log::error;
use reqwest::Client;
use serde::Deserialize;

use crate::error::{Error, Result};

#[cfg(not(target_arch = "wasm32"))]
use super::disk::from_file;
#[cfg(target_arch = "wasm32")]
use super::wasm::from_file;

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

impl ConfigFile {
    pub fn load() -> Result<ConfigFile> {
        let config = from_file()?;

        Ok(config)
    }

    pub async fn load_with_url<S>(url: S) -> Result<ConfigFile>
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
