use log::info;

use crate::{config::loader::AdoConfig, data::types::AdoData, error::Result, search::google::GoogleCSE};

use super::args::ToolArgs;

pub struct FunctionsSearch {
    search: GoogleCSE,
}

impl FunctionsSearch {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        let search = GoogleCSE::new(config)?;

        Ok(Self { search })
    }

    pub async fn search(&self, args: &ToolArgs<'_>) -> Result<AdoData> {
        let query = args.get_string("query")?;

        info!("search term: {query}");

        let data = self.search.query(query).await?;

        let b64string = args.to_base64_string(data.as_bytes())?;

        Ok(AdoData::Base64(b64string))
    }
}
