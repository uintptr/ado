use log::info;

use crate::{
    config::loader::AdoConfig,
    error::{Error, Result},
    kv::cache::KVCache,
    search::{
        google::GoogleCSE,
        results::{WebResult, WebResultEntry},
    },
};

pub(crate) mod google;

pub mod results;

pub enum WebSearch<'a> {
    Google(GoogleCSE<'a>),
}

impl<'a> WebSearch<'a> {
    pub fn new(config: &AdoConfig, cache: &'a KVCache) -> Result<Self> {
        if let Ok(google) = GoogleCSE::new(config, cache) {
            info!("Using google API");
            return Ok(WebSearch::Google(google));
        }

        Err(Error::ConfigNotFound)
    }
}

impl SearchTrait for WebSearch<'_> {
    fn query<S: AsRef<str>>(&self, query: S) -> Result<WebResult> {
        match self {
            WebSearch::Google(g) => g.query(query),
        }
    }
}

pub trait SearchTrait {
    fn lucky<S: AsRef<str>>(&self, query: S) -> Result<WebResultEntry> {
        let result = self.query(query)?;

        let first = result.entries.into_iter().next().ok_or(Error::EmptySearchResult)?;

        Ok(first)
    }
    fn query<S: AsRef<str>>(&self, query: S) -> Result<WebResult>;
}
