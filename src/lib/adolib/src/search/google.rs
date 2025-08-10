use reqwest::Client;
use serde_json::Value;

use crate::{
    config_file::loader::{ConfigFile, GoogleConfig},
    data::types::AdoDataMarkdown,
    error::{Error, Result},
};

#[derive(Debug)]
pub struct GoogleCSE {
    http_client: Client,
    google: GoogleConfig,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoogleSearchResults {
    pub json_string: String,
}

impl GoogleSearchResults {
    pub fn new(json_string: String) -> Self {
        Self { json_string }
    }
}

impl AdoDataMarkdown for GoogleSearchResults {
    fn to_markdown(self) -> crate::error::Result<String> {
        let value: Value = serde_json::from_str(&self.json_string)?;

        let items = value.get("items").ok_or(Error::InvalidFormat)?;

        let items = items.as_array().ok_or(Error::InvalidFormat)?;

        let mut md_lines = Vec::new();

        md_lines.push("# Search Results".to_string());

        for (i, item) in items.iter().enumerate() {
            let title = match item.get("title").and_then(|v| v.as_str()) {
                Some(v) => v,
                None => continue,
            };

            let link = match item.get("link").and_then(|v| v.as_str()) {
                Some(v) => v,
                None => continue,
            };

            let link_display = match item.get("displayLink").and_then(|v| v.as_str()) {
                Some(v) => v,
                None => continue,
            };

            let snippet = match item.get("snippet").and_then(|v| v.as_str()) {
                Some(v) => v,
                None => continue,
            };

            md_lines.push(format!("## {i} {title}"));
            md_lines.push(format!(" * [{link_display}]({link})"));
            md_lines.push(format!("> {snippet}"));
        }

        Ok(md_lines.join("\n"))
    }
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

        let config = ConfigFile::from_default().unwrap();

        let search = GoogleCSE::new(&config).unwrap();

        let d = search.lucky("test").await;

        info!("{d:?}");
    }
}
