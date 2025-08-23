use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strfmt::strfmt;

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

impl AdoDataMarkdown for StatusInfo {
    fn to_markdown(self) -> Result<String> {
        let mut lines = Vec::new();

        lines.push("# Status".into());

        let fmt = r#"
|             |                     |
|-------------|---------------------|
| Version     |  `{version}`        |
| Build Date  |  `{build_date}`     |
| Commit Hash |  `{commit_hash}`    |
| LLM         |  `{llm}`            |
| LLM Model   |  `{model}`          |"#;

        let mut vars: HashMap<String, String> = HashMap::new();

        vars.insert("model".into(), self.model);
        vars.insert("version".into(), self.version);
        vars.insert("build_date".into(), self.build_date);
        vars.insert("commit_hash".into(), self.commit_hash);
        vars.insert("llm".into(), self.llm_provider);

        let table = strfmt(fmt, &vars)?;

        lines.push(table);

        Ok(lines.join("\n"))
    }
}
