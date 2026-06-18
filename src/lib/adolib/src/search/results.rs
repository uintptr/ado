use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebResultEntry {
    pub title: String,
    pub link: String,
    pub link_display: String,
    pub snippet: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebResult {
    pub entries: Vec<WebResultEntry>,
}

impl FromStr for WebResult {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let results: WebResult = serde_json::from_str(s)?;
        Ok(results)
    }
}
