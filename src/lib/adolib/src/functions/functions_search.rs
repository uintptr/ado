use log::info;
use reqwest::Client;

use crate::{
    config::file::{ConfigFile, GoogleConfig},
    error::Result,
};

use super::function_args::FunctionArgs;

pub struct FunctionsSearch<'a> {
    client: Client,
    google: &'a GoogleConfig,
}

impl<'a> FunctionsSearch<'a> {
    pub fn new(config: &'a ConfigFile) -> Result<Self> {
        let google = config.search()?;

        Ok(Self {
            client: Client::new(),
            google,
        })
    }

    pub async fn search_query<S>(&self, query: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        let res = self
            .client
            .get(&self.google.url)
            .query(&[
                ("key", self.google.key.as_str()),
                ("cx", self.google.cx.as_str()),
                ("q", query.as_ref()),
                ("gl", self.google.geo.as_str()),
            ])
            .send()
            .await?;

        let body = res.text().await?;

        Ok(body)
    }

    pub async fn search(&self, args: &FunctionArgs) -> Result<String> {
        let query = args.get_string("query")?;

        info!("search term: {query}");

        let data = self.search_query(query).await?;

        args.to_base64_string(data.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::file::ConfigFile, staples::setup_logger};

    use super::FunctionsSearch;

    #[tokio::test]
    async fn search_test() {
        setup_logger(true).unwrap();

        let config = ConfigFile::load().unwrap();

        let search = FunctionsSearch::new(&config).unwrap();

        search.search_query("Hello World").await.unwrap();
    }
}
