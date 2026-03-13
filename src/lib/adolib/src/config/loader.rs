use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{
    const_vars::CONFIG_FILE_NAME,
    error::{Error, Result},
    llm::config::{ClaudeConfig, ConfigOllama},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigLlm {
    ollama: Option<ConfigOllama>,
    claude: Option<ClaudeConfig>,
    provider: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ConfigFile {
    llm: ConfigLlm,
}

#[derive(Clone)]
pub enum AdoConfigSource {
    File { path: PathBuf },
    String,
}

#[derive(Clone)]
pub struct AdoConfig {
    source: AdoConfigSource,
    config_file: ConfigFile,
}

impl ConfigFile {}

impl AdoConfig {
    fn new(source: AdoConfigSource, config_file: ConfigFile) -> Self {
        Self { source, config_file }
    }

    pub fn sync(&self) -> Result<()> {
        let toml_file = toml::to_string(&self.config_file)?;

        //
        // Update the config file
        //
        match &self.source {
            AdoConfigSource::File { path } => {
                info!("syncing {}", path.display());

                let mut fd = fs::OpenOptions::new().write(true).truncate(true).create(true).open(path)?;

                fd.lock()?;
                fd.write_all(toml_file.as_bytes())?;
            }
            AdoConfigSource::String => return Err(Error::NotImplemented),
        }

        Ok(())
    }

    pub fn from_path<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let source = AdoConfigSource::File {
            path: path.as_ref().into(),
        };

        info!("Using config file {}", path.as_ref().display());

        let file_data = match fs::read_to_string(&path) {
            Ok(v) => v,
            Err(e) => {
                error!("Unable to read config file @ {}", path.as_ref().display());
                return Err(e.into());
            }
        };

        let config_file: ConfigFile = match toml::from_str(&file_data) {
            Ok(v) => v,
            Err(e) => {
                error!("Unable to deserialize {e}");
                return Err(e.into());
            }
        };

        Ok(AdoConfig::new(source, config_file))
    }

    pub fn from_default() -> Result<Self> {
        let config_dir = dirs::config_dir().ok_or(Error::ConfigNotFound)?;

        let ado_config_dir = config_dir.join("ado");

        if !ado_config_dir.exists() {
            fs::create_dir_all(&ado_config_dir)?;
        }

        let config_file = ado_config_dir.join(CONFIG_FILE_NAME);

        AdoConfig::from_path(config_file)
    }

    // mainly only used in testing
    pub fn from_string<S>(value: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let config_file: ConfigFile = toml::from_str(value.as_ref())?;

        Ok(AdoConfig::new(AdoConfigSource::String, config_file))
    }

    #[must_use]
    pub fn llm_provider(&self) -> &str {
        &self.config_file.llm.provider
    }

    pub fn llm_provider_update<S>(&mut self, llm: S)
    where
        S: AsRef<str>,
    {
        self.config_file.llm.provider = llm.as_ref().to_string();
    }

    pub fn ollama(&self) -> Result<&ConfigOllama> {
        match &self.config_file.llm.ollama {
            Some(v) => Ok(v),
            None => Err(Error::ConfigNotFound),
        }
    }

    pub fn claude(&self) -> Result<&ClaudeConfig> {
        match &self.config_file.llm.claude {
            Some(v) => Ok(v),
            None => Err(Error::ConfigNotFound),
        }
    }
}
