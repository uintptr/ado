use async_trait::async_trait;
use log::info;
use omcp::{client::types::BakedMcpToolTrait, types::McpParams};
use reqwest::Client;
use whois_rust::{WhoIs, WhoIsLookupOptions};

use crate::{
    error::{Error, Result},
    mcp::assets::McpWhoisAssets,
};

pub struct ToolGetIpAddress {}
pub struct ToolWhoisQuery {
    provider: WhoIs,
}

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
    pub fn new() -> Result<Self> {
        let config_file = McpWhoisAssets::get("whois_servers.json").ok_or(Error::FileNotFoundError {
            file_path: "whois_servers.json".into(),
        })?;

        let config_string = String::from_utf8(config_file.data.to_vec())?;

        let provider = WhoIs::from_string(config_string)?;

        Ok(Self { provider })
    }

    pub fn query_domain(&self, domain_name: &str) -> Result<String> {
        info!("looking for domain_name={domain_name}");

        let opts = WhoIsLookupOptions::from_str(domain_name)?;

        let data = self.provider.lookup(opts)?;

        Ok(data)
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolWhoisQuery {
    type Error = Error;

    async fn call(&mut self, params: &McpParams) -> Result<String> {
        let domain_name = params.get_string("domain_name")?;

        info!("domain_name={domain_name}");

        let response = self.query_domain(domain_name)?;

        Ok(response)
    }
}
