use std::{fs, path::Path};

use serde::Deserialize;

use crate::{
    error::{Error, Result},
    staples::find_file,
};

const CONFIG_FILE: &str = "config.toml";

#[derive(Deserialize)]
pub struct AdoGemini {
    pub key: String,
    pub url: String,
}

#[derive(Deserialize)]
pub struct OpenAiConfig {
    pub key: String,
    #[serde(default = "openai_default_url")]
    pub url: String,
    #[serde(default = "openai_default_model")]
    pub model: String,
}

#[derive(Deserialize)]
pub enum AdoConfig {
    #[serde(rename = "gemini")]
    Gemini(AdoGemini),
    #[serde(rename = "openai")]
    Openai(OpenAiConfig),
}

fn openai_default_url() -> String {
    "https://api.openai.com/v1/responses".to_string()
}

fn openai_default_model() -> String {
    "gpt-4.1".to_string()
}

impl AdoConfig {
    pub fn load() -> Result<AdoConfig> {
        let rel_config = Path::new("config").join(CONFIG_FILE);

        let config_file = find_file(rel_config)?;

        let file_data = fs::read_to_string(config_file)?;

        let config: AdoConfig = toml::from_str(&file_data)?;

        Ok(config)
    }

    pub fn gemini(&self) -> Result<&AdoGemini> {
        match self {
            AdoConfig::Gemini(g) => Ok(g),
            _ => Err(Error::LlmNotFound {
                llm: "gemini".to_string(),
            }),
        }
    }

    pub fn openai(self) -> Result<OpenAiConfig> {
        match self {
            AdoConfig::Openai(o) => Ok(o),
            _ => Err(Error::LlmNotFound {
                llm: "openai".to_string(),
            }),
        }
    }
}
