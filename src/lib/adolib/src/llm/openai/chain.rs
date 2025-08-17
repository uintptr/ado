use log::info;

use crate::{
    config::loader::AdoConfig,
    data::types::AdoData,
    error::Result,
    llm::{openai::api::OpenAIAPI, provider::LLMChainTrait},
    tools::config::ConfigFunctions,
};

use async_trait::async_trait;

use super::request::OpenAIRequest;

const FUNC_PROMPT_PRE: &str = r#"Dont forget that you have access series of
tools and functions to call to give the user the best possible answer. Here's
the list of functions"#;

pub struct OpenAIChain {
    api: OpenAIAPI,
    req: OpenAIRequest,
}

fn build_functions_prompt(functions: &ConfigFunctions) -> String {
    let mut func_names: Vec<&str> = Vec::new();

    for f in functions.list.iter() {
        func_names.push(&f.name);
    }

    let func_names_str = func_names.join(",");

    format!("{FUNC_PROMPT_PRE}: {func_names_str}")
}

impl OpenAIChain {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        let functions = ConfigFunctions::load()?;

        let api = OpenAIAPI::new(config)?;

        let openai = config.openai()?;

        let function_prompt = build_functions_prompt(&functions);

        let mut req = OpenAIRequest::new(&openai.model);

        req.with_functions(functions);

        if let Some(prompt) = &openai.prompt {
            req.with_input_role("user", prompt);
        }

        req.with_input_role("user", function_prompt);

        Ok(Self { api, req })
    }
}

#[async_trait(?Send)]
impl LLMChainTrait for OpenAIChain {
    async fn query(&mut self, content: &str) -> Result<AdoData> {
        info!("query: {}", content);
        self.req.with_input_role("user", content);
        self.api.query(&mut self.req).await
    }

    async fn message(&self, content: &str) -> Result<String> {
        self.api.message(content).await
    }

    fn reset(&mut self) {
        self.req.reset_input();
    }

    fn model(&self) -> &str {
        self.api.model()
    }
}
