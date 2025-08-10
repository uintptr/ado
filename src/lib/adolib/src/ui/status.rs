use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strfmt::strfmt;

use crate::{
    const_vars::{PKG_VERSION, VERGEN_BUILD_DATE, VERGEN_RUSTC_COMMIT_HASH},
    data::types::AdoDataMarkdown,
    error::Result,
    llm::openai::chain::AIChain,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

        let fmt = r#"
|             |                   |
|-------------|-------------------|
| Model       |  `{model}`        |
| Version     |  `{version}`      |
| Build Date  |  `{build_date}`   |
| Commit Hash |  `{commit_hash}`  |"#;

        let mut vars: HashMap<String, String> = HashMap::new();

        vars.insert("model".into(), self.model);
        vars.insert("version".into(), self.version);
        vars.insert("build_date".into(), self.build_date);
        vars.insert("commit_hash".into(), self.commit_hash);

        let table = strfmt(fmt, &vars)?;

        lines.push(table);

        Ok(lines.join("\n"))
    }
}
