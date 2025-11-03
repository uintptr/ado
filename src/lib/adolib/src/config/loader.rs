use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{
    const_vars::{CONFIG_FILE_NAME, DIRS_APP, DIRS_ORG, DIRS_QUALIFIER, STORE_PERMANENT},
    error::{Error, Result},
    llm::config::{ClaudeConfig, ConfigOllama},
    mcp::types::McpConfig,
    search::google::GoogleConfig,
    storage::{PersistentStorageTrait, persistent::PersistentStorage},
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
    search: Option<GoogleConfig>,
    mcp: Option<HashMap<String, McpConfig>>,
}

#[derive(Clone)]
pub enum AdoConfigSource {
    File { path: PathBuf },
    Webdis { storage: PersistentStorage },
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

    pub async fn sync(&self) -> Result<()> {
        let toml_file = toml::to_string(&self.config_file)?;

        //
        // Update the config file
        //
        match &self.source {
            AdoConfigSource::File { path } => {
                info!("syncing {}", path.display());
                fs::write(path, toml_file.as_bytes())?
            }
            AdoConfigSource::Webdis { storage } => storage.set("global", "config", toml_file, STORE_PERMANENT).await?,
            _ => return Err(Error::NotImplemented),
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

        let config_file: ConfigFile = toml::from_str(&file_data)?;

        Ok(AdoConfig::new(source, config_file))
    }

    pub fn from_default() -> Result<Self> {
        let dirs = ProjectDirs::from(DIRS_QUALIFIER, DIRS_ORG, DIRS_APP).ok_or(Error::NotFound)?;

        let config_dir = dirs.config_dir();

        if config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }

        let config_file = config_dir.join(CONFIG_FILE_NAME);

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

    #[cfg(target_arch = "wasm32")]
    pub async fn from_webdis<U, S>(user_id: U, server: S) -> Result<Self>
    where
        U: AsRef<str>,
        S: AsRef<str>,
    {
        let storage = PersistentStorage::new(&user_id, server);

        let data = storage.get("global", "config").await?;

        let source = AdoConfigSource::Webdis { storage: storage };

        let config_file: ConfigFile = toml::from_str(&data)?;

        Ok(AdoConfig::new(source, config_file))
    }

    pub fn llm_provider(&self) -> &str {
        &self.config_file.llm.provider
    }

    pub fn update_llm<S>(&mut self, llm: S)
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

    pub fn search(&self) -> Result<&GoogleConfig> {
        match &self.config_file.search {
            Some(v) => Ok(v),
            None => Err(Error::ConfigNotFound),
        }
    }

    pub fn mcp_servers(&self) -> Option<&HashMap<String, McpConfig>> {
        match &self.config_file.mcp {
            Some(v) => Some(v),
            None => None,
        }
    }
}
