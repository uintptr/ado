use log::info;

use crate::{config::file::ConfigFile, data::AdoData, error::Result, functions::config::ConfigFunctions};

use super::{api::LLM, request::OpenAIRequest};

const FUNC_PROMPT_PRE: &str = r#"Dont forget that you have access series of
tools and functions to call to give the user the best possible answer. Here's
the list of functions"#;

pub struct AIChain {
    llm: LLM,
    req: OpenAIRequest,
}

fn build_functions_prompt(functions: &ConfigFunctions) -> String {
    let mut func_names: Vec<&str> = Vec::new();

    for f in functions.list.iter() {
        func_names.push(&f.name);
    }

    let func_names_str = func_names.join(",");

    format!("{}: {}", FUNC_PROMPT_PRE, func_names_str)
}

impl AIChain {
    pub fn new(config: &ConfigFile) -> Result<AIChain> {
        let functions = ConfigFunctions::load()?;

        let llm = LLM::new(config)?;

        let openai = config.openai()?;

        let function_prompt = build_functions_prompt(&functions);

        let mut req = OpenAIRequest::new(&openai.model);

        req.with_functions(functions);

        if let Some(prompt) = &openai.prompt {
            req.with_input_role("user", prompt);
        }

        req.with_input_role("user", function_prompt);

        Ok(AIChain { llm, req })
    }

    pub async fn query<S>(&mut self, content: S) -> Result<Vec<AdoData>>
    where
        S: AsRef<str>,
    {
        info!("query: {}", content.as_ref());
        self.req.with_input_role("user", content);
        self.llm.query(&mut self.req).await
    }

    pub fn reset(&mut self) {
        self.req.reset_input();
    }
}
