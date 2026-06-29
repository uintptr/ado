use std::{fmt::Display, str::FromStr};

use log::error;
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

impl Display for WebResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match serde_json::to_string(self) {
            Ok(v) => v,
            Err(e) => {
                error!("unable deserialized ({e})");
                "{}".to_string()
            }
        };
        write!(f, "{s}")
    }
}
