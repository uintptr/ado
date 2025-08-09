use serde::{Deserialize, Serialize};

use crate::{
    const_vars::{PKG_VERSION, VERGEN_BUILD_DATE, VERGEN_RUSTC_COMMIT_HASH},
    data::types::AdoDataMarkdown,
    error::Result,
    llm::openai::chain::AIChain,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusInfo {
    pub model: String,
    pub version: String,
    pub build_date: String,
    pub commit_hash: String,
}

impl StatusInfo {
    pub fn new(chain: &AIChain) -> Self {
        let model = chain.model();

        StatusInfo {
            model: model.to_string(),
            version: PKG_VERSION.into(),
            build_date: VERGEN_BUILD_DATE.into(),
            commit_hash: VERGEN_RUSTC_COMMIT_HASH.into(),
        }
    }
}

impl AdoDataMarkdown for StatusInfo {
    fn to_markdown(self) -> Result<String> {
        let mut lines = Vec::new();

        lines.push("# Status".into());
        lines.push(format!("*  LLM Model: `{}`", self.model));
        lines.push(format!("*  Version: `{}`", self.version));
        lines.push(format!("*  Build Date: `{}`", self.build_date));
        lines.push(format!("*  Commit Hash: `{}`", self.commit_hash));

        Ok(lines.join("\n"))
    }
}
