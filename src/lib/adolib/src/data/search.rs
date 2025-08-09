use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{data::types::AdoDataMarkdown, error::Error};

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleSearchData {
    pub json_string: String,
}

impl GoogleSearchData {
    pub fn new(json_string: String) -> Self {
        Self { json_string }
    }
}

impl AdoDataMarkdown for GoogleSearchData {
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
