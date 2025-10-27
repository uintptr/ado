use async_trait::async_trait;
use log::info;
use omcp::types::{BakedMcpToolTrait, McpParams};

use crate::{
    config::loader::AdoConfig,
    error::{Error, Result},
    search::google::GoogleCSE,
};

pub struct ToolWebSearch {
    google: GoogleCSE,
}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////
// WHOIS
///////////////////////////////////////

impl ToolWebSearch {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        let google = GoogleCSE::new(config)?;
        Ok(Self { google })
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolWebSearch {
    type Error = Error;

    async fn call(&mut self, params: &McpParams) -> Result<String> {
        let query = params.get_string("query")?;

        info!("search query {query}");

        let data = self.google.query(query).await?;

        Ok(data)
    }
}
