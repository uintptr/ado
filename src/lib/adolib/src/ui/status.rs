use serde::{Deserialize, Serialize};

use crate::{
    config::loader::AdoConfig,
    const_vars::{LIB_VERSION, VERGEN_BUILD_DATE, VERGEN_RUSTC_COMMIT_HASH},
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
    #[must_use]
    pub fn new(config_file: &AdoConfig, chain: &LLMChain) -> Self {
        let model = chain.model();

        StatusInfo {
            model: model.to_string(),
            version: LIB_VERSION.into(),
            build_date: VERGEN_BUILD_DATE.into(),
            commit_hash: VERGEN_RUSTC_COMMIT_HASH.into(),
            llm_provider: config_file.llm_provider().to_string(),
        }
    }
}
