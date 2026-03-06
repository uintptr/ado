use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigOllama {
    pub endpoint: String,
    pub model: String,
    #[serde(default = "default_false")] // defaults to true
    pub thinking: bool,
}

fn default_false() -> bool {
    false
}
