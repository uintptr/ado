use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{data::types::AdoDataMarkdown, error::Error};

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

impl AdoDataMarkdown for &WebResult {
    fn to_markdown(self) -> crate::error::Result<String> {
        let mut md_lines = Vec::new();

        md_lines.push("# Search Results".to_string());

        for (i, item) in self.entries.iter().enumerate() {
            md_lines.push(format!("## {i} {}", item.title));
            md_lines.push(format!(" * [{}]({})", item.link_display, item.link));
            md_lines.push(format!("> {}", item.snippet));
        }

        Ok(md_lines.join("\n"))
    }
}
