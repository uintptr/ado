use std::{fs, path::Path};

use serde::Deserialize;

use crate::{error::Result, staples::find_file};

const CONFIG_FILE: &str = "config.toml";

#[derive(Deserialize)]
pub struct AdoGemini {
    pub key: String,
    pub url: String,
}

#[derive(Deserialize)]
pub enum AdoConfig {
    #[serde(rename = "gemini")]
    Gemini(AdoGemini),
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
        }
    }
}
