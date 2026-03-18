use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigOllama {
    pub endpoint: String,
    pub model: String,
    #[serde(default = "default_false")] // defaults to true
    pub thinking: bool,
    #[serde(default = "default_keep_alive")] // defaults to true
    pub keep_alive: i32,
}

fn default_false() -> bool {
    false
}

fn default_keep_alive() -> i32 {
    1800
}
