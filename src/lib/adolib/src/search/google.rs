use std::time::Duration;

use log::{error, info};
use moka::sync::Cache;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoogleConfig {
    pub cx: String,
    pub geo: String,
    pub key: String,
    pub url: String,
    pub cache_size: u64,
    pub cache_ttl: u64,
}

#[derive(Debug)]
pub struct GoogleCSE {
    google: GoogleConfig,
    cache: Cache<String, String>,
}

use crate::{
    config::loader::AdoConfig,
    error::{Error, Result},
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

impl SearchTrait for GoogleCSE {
    fn query<S: AsRef<str>>(&self, query: S) -> Result<WebResult> {
        let json_data = self.query_layerd(query)?;
        parse_results(&json_data)
    }
}

impl GoogleCSE {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        let google = config.search_google()?.clone();

        let cache = Cache::builder()
            .max_capacity(google.cache_size)
            .time_to_live(Duration::from_secs(google.cache_ttl))
            .build();

        Ok(Self { google, cache })
    }

    fn query_cached<S: AsRef<str>>(&self, query: S) -> Option<String> {
        self.cache.get(query.as_ref())
    }

    fn query_layerd<S: AsRef<str>>(&self, query: S) -> Result<String> {
        if let Some(cached) = self.query_cached(&query) {
            info!("{} was cached", query.as_ref());
            return Ok(cached);
        }

        let ret = self.query_remote(&query);

        if let Ok(data) = &ret {
            self.cache.insert(query.as_ref().into(), data.clone());
        }

        ret
    }

    fn query_remote<S: AsRef<str>>(&self, query: S) -> Result<String> {
        let query = vec![
            ("key", self.google.key.as_str()),
            ("cx", self.google.cx.as_str()),
            ("q", query.as_ref()),
            ("gl", self.google.geo.as_str()),
        ];

        let mut res = ureq::get(&self.google.url).query_pairs(query).call()?;

        if !res.status().is_success() {
            error!("{} returned {}", self.google.url, res.status().as_str());
            return Err(Error::HttpGetFailure);
        }

        let body = res.body_mut().read_to_string()?;

        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use log::info;

    use crate::config::loader::AdoConfig;

    use super::*;

    #[test]
    fn test_lucky() {
        env_logger::init();

        let config = AdoConfig::from_default().unwrap();

        let search = GoogleCSE::new(&config).unwrap();

        let d = search.lucky("test");

        info!("{d:?}");
    }

    #[test]
    fn test_parse_results() {
        env_logger::init();

        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let test_file = root.join("test").join("search_test.json");

        let data = fs::read_to_string(test_file).unwrap();

        let result = parse_results(&data).unwrap();
        info!("{result:?}");
    }
}
