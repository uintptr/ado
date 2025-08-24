use std::fs;

use serde::{Deserialize, Serialize};

use crate::{
    config::loader::AdoConfig,
    const_vars::{PKG_VERSION, VERGEN_BUILD_DATE, VERGEN_RUSTC_COMMIT_HASH},
    data::types::AdoDataMarkdown,
    error::Result,
    llm::chain::LLMChain,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusInfo {
    pub model: String,
    pub version: String,
    pub build_date: String,
    pub commit_hash: String,
    pub llm_provider: String,
}

impl StatusInfo {
    pub fn new(config_file: &AdoConfig, chain: &LLMChain) -> Self {
        let model = chain.model();

        StatusInfo {
            model: model.to_string(),
            version: PKG_VERSION.into(),
            build_date: VERGEN_BUILD_DATE.into(),
            commit_hash: VERGEN_RUSTC_COMMIT_HASH.into(),
            llm_provider: config_file.llm_provider().to_string(),
        }
    }
}

impl AdoDataMarkdown for &StatusInfo {
    fn to_markdown(self) -> Result<String> {
        let table = format!(
            r#"
|             |                     |
|-------------|---------------------|
| Version     |  `{}` |
| Build Date  |  `{}` |
| Commit Hash |  `{}` |
| LLM         |  `{}` |
| LLM Model   |  `{}` |"#,
            self.version, self.build_date, self.commit_hash, self.llm_provider, self.model
        );

        fs::write("/tmp/table.md", table.as_bytes())?;

        Ok(table)
    }
}
