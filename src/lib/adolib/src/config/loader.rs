use std::{
    env, fs,
    path::{Path, PathBuf},
};

use log::{error, info};
use rstaples::staples::find_file;
use serde::{Deserialize, Serialize};

use crate::{
    const_vars::{DOT_DIRECTORY, STORE_PERMANENT},
    error::{Error, Result},
    storage::{PersistentStorageTrait, persistent::PersistentStorage},
};

const DEF_OPENAI_URL: &str = "https://api.openai.com/v1/responses";
const DEF_OPENAI_MODEL: &str = "gpt-5-mini";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAiConfig {
    #[serde(default = "openai_default_key")]
    pub key: String,
    #[serde(default = "openai_default_url")]
    pub url: String,
    #[serde(default = "openai_default_model")]
    pub model: String,
    pub instructions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaudeAiConfig {
    pub model: String,
    pub url: String,
    pub anthropic_version: String,
    pub key: String,
    pub max_tokens: u64,
    pub instructions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigLlmLlama {
    pub endpoint: String,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoogleConfig {
    pub cx: String,
    pub geo: String,
    pub key: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigLlm {
    openai: Option<OpenAiConfig>,
    ollama: Option<ConfigLlmLlama>,
    claude: Option<ClaudeAiConfig>,
    provider: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ConfigFile {
    llm: ConfigLlm,
    search: Option<GoogleConfig>,
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

impl ConfigFile {}

impl AdoConfig {
    fn new(source: AdoConfigSource, config_file: ConfigFile) -> Self {
        Self { source, config_file }
    }

    pub async fn sync(&self) -> Result<()> {
        let toml_file = toml::to_string(&self.config_file)?;

        //
        // Update th config file
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

        let file_data = fs::read_to_string(&path)?;
        let config_file: ConfigFile = toml::from_str(&file_data)?;

        Ok(AdoConfig::new(source, config_file))
    }

    pub fn from_default() -> Result<Self> {
        let rel_config = Path::new("config").join(CONFIG_FILE_NAME);

        let config_file = match find_file(rel_config) {
            Ok(v) => v,
            Err(_) => find_from_home()?,
        };

        AdoConfig::from_path(config_file)
    }

    // only used for testing
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

    pub fn ollama(&self) -> Result<&ConfigLlmLlama> {
        match &self.config_file.llm.ollama {
            Some(v) => Ok(v),
            None => Err(Error::ConfigNotFound),
        }
    }

    pub fn openai(&self) -> Result<&OpenAiConfig> {
        match &self.config_file.llm.openai {
            Some(v) => Ok(v),
            None => Err(Error::ConfigNotFound),
        }
    }

    pub fn claude(&self) -> Result<&ClaudeAiConfig> {
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
}
