use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigOllama {
    pub endpoint: String,
    pub model: String,
}
