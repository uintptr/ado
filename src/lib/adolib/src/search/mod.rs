use async_trait::async_trait;

use crate::{
    error::{Error, Result},
    search::results::{WebResult, WebResultEntry},
};

pub(crate) mod google;
pub(crate) mod serp;

pub mod results;

pub use google::GoogleCSE as WebSearch;

#[async_trait(?Send)]
pub trait SearchTrait {
    async fn lucky<S: AsRef<str>>(&self, query: S) -> Result<WebResultEntry> {
        let result = self.query(query).await?;

        let first = result.entries.into_iter().next().ok_or(Error::EmptySearchResult)?;

        Ok(first)
    }
    async fn query<S: AsRef<str>>(&self, query: S) -> Result<WebResult>;
}
