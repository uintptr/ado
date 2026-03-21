use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    config::loader::AdoConfig,
    error::{Error, Result},
    search::{
        SearchTrait,
        results::{WebResult, WebResultEntry},
    },
};

#[derive(Deserialize)]
struct SerpOrganicResult {
    title: String,
    link: String,
    snippet: String,
    displayed_link: String,
}

#[derive(Deserialize)]
struct SerpResponse {
    organic_results: Vec<SerpOrganicResult>,
}

impl From<SerpResponse> for WebResult {
    fn from(value: SerpResponse) -> Self {
        let mut entries = Vec::new();

        for serp_entry in value.organic_results {
            let entry = WebResultEntry {
                title: serp_entry.title,
                link: serp_entry.link,
                link_display: serp_entry.displayed_link,
                snippet: serp_entry.snippet,
            };

            entries.push(entry);
        }

        WebResult { entries }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerpApiConfig {
    pub engine: String,
    pub location: String,
    pub url: String,
    pub google_domain: String,
    pub hl: String,
    pub gl: String,
    pub api_keys: Vec<String>,
}

// see https://serpapi.com/search-api
pub struct SerpApi {
    config: SerpApiConfig,
    client: Client,
}

fn parse_results(data: &str) -> Result<WebResult> {
    let serp_result: SerpResponse = serde_json::from_str(data)?;
    Ok(serp_result.into())
}

impl SerpApi {
    pub fn new(ado_config: &AdoConfig) -> Result<Self> {
        let config = ado_config.search_setp()?.clone();
        let client = reqwest::Client::new();

        Ok(Self { config, client })
    }
}

#[async_trait(?Send)]
impl SearchTrait for SerpApi {
    async fn query<S: AsRef<str>>(&self, query: S) -> Result<WebResult> {
        let mut args = HashMap::new();

        #[cfg(not(target_arch = "wasm32"))]
        let api_key = {
            use rand::seq::IndexedRandom;
            let mut rng = rand::rng();
            self.config.api_keys.choose(&mut rng).ok_or(Error::ApiKeyNotFound)?
        };
        #[cfg(target_arch = "wasm32")]
        let api_key = self.config.api_keys.first().ok_or(Error::ApiKeyNotFound)?;

        args.insert("engine", self.config.engine.as_str());
        args.insert("q", query.as_ref());
        args.insert("location", self.config.location.as_str());
        args.insert("google_domain", self.config.google_domain.as_str());
        args.insert("hl", self.config.hl.as_str());
        args.insert("gl", self.config.gl.as_str());
        args.insert("api_key", api_key.as_str());
        args.insert("json_restrictor", "organic_results,search_parameters");

        let res = self.client.get(&self.config.url).query(&args).send().await?;

        dbg!(&res);

        let body = res.text().await?;

        println!("{body}");

        parse_results(&body)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use log::info;

    use crate::{logging::logger::setup_logger, search::serp::parse_results};

    use super::*;

    #[tokio::test]
    async fn test_lucky() {
        setup_logger(true).unwrap();

        let config = AdoConfig::from_default().unwrap();

        let search = SerpApi::new(&config).unwrap();

        let d = search.lucky("test").await;

        info!("{d:?}");
    }

    #[test]
    fn test_parse_results() {
        setup_logger(true).unwrap();

        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let test_file = root.join("test").join("serp_results.json");

        let data = fs::read_to_string(test_file).unwrap();

        parse_results(&data).unwrap();
    }
}
