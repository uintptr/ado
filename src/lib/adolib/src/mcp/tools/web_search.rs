use async_trait::async_trait;
use log::info;
use omcp::types::{BakedMcpToolTrait, McpParams};

use crate::{
    config::loader::AdoConfig,
    error::{Error, Result},
    search::{SearchTrait, WebSearch},
};

pub struct ToolWebSearch {
    web_search: WebSearch,
}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////
// WHOIS
///////////////////////////////////////

impl ToolWebSearch {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        let web_search = WebSearch::new(config)?;
        Ok(Self { web_search })
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolWebSearch {
    type Error = Error;

    async fn call(&mut self, params: &McpParams) -> Result<String> {
        let query = params.get_string("query")?;

        info!("search query {query}");

        let data = self.web_search.query(query).await?;
        let data = serde_json::to_string_pretty(&data)?;

        Ok(data)
    }
}
