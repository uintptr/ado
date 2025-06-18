use reqwest::Client;
use serde_json::Value;

use crate::{
    config::file::{ConfigFile, GoogleConfig},
    error::{Error, Result},
};

#[derive(Debug)]
pub struct GoogleCSE {
    http_client: Client,
    google: GoogleConfig,
}

impl GoogleCSE {
    pub fn new(config: &ConfigFile) -> Result<Self> {
        let google = config.search()?.clone();

        Ok(Self {
            http_client: Client::new(),
            google,
        })
    }

    pub async fn lucky<S>(&self, query: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        let data = self.query(query).await?;

        let dict: Value = serde_json::from_str(&data)?;

        let items = dict.get("items").ok_or(Error::EmptySearchResult)?;

        let first = items.as_array().ok_or(Error::InvalidJsonType)?.first().ok_or(Error::Empty)?;

        let link = first.get("link").ok_or(Error::Empty)?;

        let link = link.as_str().ok_or(Error::InvalidJsonType)?;

        Ok(link.into())
    }

    pub async fn query<S>(&self, query: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        let res = self
            .http_client
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
}

#[cfg(test)]
mod tests {
    use log::info;

    use crate::logging::logger::setup_logger;

    use super::*;

    #[tokio::test]
    async fn test_lucky() {
        setup_logger(true).unwrap();

        let config = ConfigFile::load().unwrap();

        let search = GoogleCSE::new(&config).unwrap();

        let d = search.lucky("test").await;

        info!("{d:?}");
    }
}
