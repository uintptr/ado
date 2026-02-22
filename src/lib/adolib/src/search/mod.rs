use async_trait::async_trait;
use log::info;

use crate::{
    config::loader::AdoConfig,
    error::{Error, Result},
    search::{
        google::GoogleCSE,
        results::{WebResult, WebResultEntry},
        serp::SerpApi,
    },
};

pub(crate) mod google;
pub(crate) mod serp;

pub mod results;

pub enum WebSearch {
    Google(GoogleCSE),
    Serp(SerpApi),
}

impl WebSearch {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        if let Ok(google) = GoogleCSE::new(config) {
            info!("Using google API");
            return Ok(WebSearch::Google(google));
        }

        if let Ok(serp) = SerpApi::new(config) {
            info!("Using Serp API");
            return Ok(WebSearch::Serp(serp));
        }

        Err(Error::ConfigNotFound)
    }
}

#[async_trait(?Send)]
impl SearchTrait for WebSearch {
    async fn query<S: AsRef<str>>(&self, query: S) -> Result<WebResult> {
        match self {
            WebSearch::Google(g) => g.query(query).await,
            WebSearch::Serp(s) => s.query(query).await,
        }
    }
}

#[async_trait(?Send)]
pub trait SearchTrait {
    async fn lucky<S: AsRef<str>>(&self, query: S) -> Result<WebResultEntry> {
        let result = self.query(query).await?;

        let first = result.entries.into_iter().next().ok_or(Error::EmptySearchResult)?;

        Ok(first)
    }
    async fn query<S: AsRef<str>>(&self, query: S) -> Result<WebResult>;
}
