use log::info;

use crate::{
    config::loader::AdoConfig,
    error::Result,
    llm::{chain::LLMChainTrait, openai::openai_api::OpenAIAPI},
    tools::loader::Tools,
    ui::ConsoleDisplayTrait,
};

use async_trait::async_trait;

use super::openai_request::OpenAIRequest;

const FUNC_PROMPT_PRE: &str = r#"Dont forget that you have access series of
tools and functions to call to give the user the best possible answer. Here's
the list of functions"#;

pub struct OpenAIChain {
    api: OpenAIAPI,
    req: OpenAIRequest,
}

fn build_functions_prompt(functions: &Tools) -> String {
    let mut func_names: Vec<&str> = Vec::new();

    for f in functions.list.iter() {
        func_names.push(&f.name);
    }

    let func_names_str = func_names.join(",");

    format!("{FUNC_PROMPT_PRE}: {func_names_str}")
}

impl OpenAIChain {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        let functions = Tools::load()?;

        let api = OpenAIAPI::new(config)?;

        let openai = config.openai()?;

        let function_prompt = build_functions_prompt(&functions);

        let mut req = OpenAIRequest::new(&openai.model);

        req.with_functions(functions);

        if let Some(instructions) = &openai.instructions {
            for i in instructions {
                req.with_input_role("system", i)
            }
        }

        req.with_input_role("user", function_prompt);

        Ok(Self { api, req })
    }
}

#[async_trait(?Send)]
impl LLMChainTrait for OpenAIChain {
    async fn link<C>(&mut self, content: &str, _console: &mut C) -> Result<()>
    where
        C: ConsoleDisplayTrait,
    {
        info!("query: {}", content);
        self.req.with_input_role("user", content);
        self.api.query(&mut self.req).await?;
        Ok(())
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

    fn change_model<S>(&mut self, model: S)
    where
        S: AsRef<str>,
    {
        self.api.config.model = model.as_ref().into()
    }
}
