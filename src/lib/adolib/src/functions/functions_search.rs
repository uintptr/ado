use log::info;

use crate::{config::file::ConfigFile, error::Result, search::google::GoogleCSE};

use super::function_args::FunctionArgs;

pub struct FunctionsSearch {
    search: GoogleCSE,
}

impl FunctionsSearch {
    pub fn new(config: &ConfigFile) -> Result<Self> {
        let search = GoogleCSE::new(config)?;

        Ok(Self { search })
    }

    pub async fn search(&self, args: &FunctionArgs) -> Result<String> {
        let query = args.get_string("query")?;

        info!("search term: {query}");

        let data = self.search.query(query).await?;

        args.to_base64_string(data.as_bytes())
    }
}
