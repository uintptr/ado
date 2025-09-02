use omcp::types::McpTypes;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct McpConfig {
    #[serde(rename = "type")]
    pub config_type: McpTypes,
    pub url: Option<String>,
    pub authorization_token: Option<String>,
}
