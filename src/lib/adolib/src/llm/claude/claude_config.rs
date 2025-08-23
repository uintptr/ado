use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub enum ClaudeToolChoiceType {
    #[default]
    #[serde(rename = "any")]
    Any,
}
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ClaudeToolChoice {
    #[serde(rename = "type")]
    pub choice_type: ClaudeToolChoiceType,
    pub disable_parallel_tool_use: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClaudeMcpServerType {
    #[serde(rename = "url")]
    Url,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaudeMcpServer {
    name: String,
    #[serde(rename = "type")]
    server_type: ClaudeMcpServerType,
    url: String,
    authorization_token: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ClaudeConfig {
    pub model: String,
    pub url: String,
    pub anthropic_version: String,
    pub key: String,
    pub max_tokens: u64,
    pub instructions: Option<Vec<String>>,
    pub mcp_servers: Option<Vec<ClaudeMcpServer>>,
    pub tool_choice: Option<ClaudeToolChoice>,
}
