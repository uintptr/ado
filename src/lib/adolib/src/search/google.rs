use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoogleConfig {
    pub cx: String,
    pub geo: String,
    pub key: String,
    pub url: String,
}

#[derive(Debug)]
pub struct GoogleCSE {
    http_client: Client,
    google: GoogleConfig,
}

use crate::{
    config::loader::AdoConfig,
    error::Result,
    search::{
        SearchTrait,
        results::{WebResult, WebResultEntry},
    },
};

#[derive(Deserialize)]
struct GoogleItem {
    title: String,
    link: String,
    #[serde(rename = "displayLink")]
    display_link: String,
    snippet: String,
}

#[derive(Deserialize)]
struct GoogleResponse {
    items: Vec<GoogleItem>,
}

fn parse_results(data: &str) -> Result<WebResult> {
    let response: GoogleResponse = serde_json::from_str(data)?;
    let entries = response
        .items
        .into_iter()
        .map(|item| WebResultEntry {
            title: item.title,
            link: item.link,
            link_display: item.display_link,
            snippet: item.snippet,
        })
        .collect();
    Ok(WebResult { entries })
}

#[async_trait(?Send)]
impl SearchTrait for GoogleCSE {
    async fn query<S: AsRef<str>>(&self, query: S) -> Result<WebResult> {
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

        parse_results(&body)
    }
}

impl GoogleCSE {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        let google = config.search_google()?.clone();

        Ok(Self {
            http_client: Client::new(),
            google,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use log::info;

    use crate::{config::loader::AdoConfig, logging::logger::setup_logger};

    use super::*;

    #[tokio::test]
    async fn test_lucky() {
        setup_logger(true).unwrap();

        let config = AdoConfig::from_default().unwrap();

        let search = GoogleCSE::new(&config).unwrap();

        let d = search.lucky("test").await;

        info!("{d:?}");
    }

    #[test]
    fn test_parse_results() {
        setup_logger(true).unwrap();

        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let test_file = root.join("test").join("search_test.json");

        let data = fs::read_to_string(test_file).unwrap();

        let result = parse_results(&data).unwrap();
        info!("{result:?}");
    }
}
