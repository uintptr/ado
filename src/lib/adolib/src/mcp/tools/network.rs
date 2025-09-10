use async_trait::async_trait;
use log::info;
use omcp::{client::types::BakedMcpToolTrait, types::McpParams};
use reqwest::Client;

use crate::error::{Error, Result};

pub struct ToolGetIpAddress {}
pub struct ToolWhoisQuery {}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////
// GET IP
///////////////////////////////////////

const API_URL: &str = "https://api.ipify.org?format=json";

impl ToolGetIpAddress {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolGetIpAddress {
    type Error = Error;

    async fn call(&mut self, _params: &McpParams) -> Result<String> {
        let client = Client::new();

        let res = client.get(API_URL).send().await?;

        if !res.status().is_success() {
            return Err(Error::ApiFailure {
                message: res.status().as_str().to_string(),
            });
        }

        let data = res.text().await?;

        Ok(data)
    }
}

///////////////////////////////////////
// WHOIS
///////////////////////////////////////

impl ToolWhoisQuery {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolWhoisQuery {
    type Error = Error;

    async fn call(&mut self, params: &McpParams) -> Result<String> {
        info!("Hello from {}", params.tool_name);
        Ok("".into())
    }
}
