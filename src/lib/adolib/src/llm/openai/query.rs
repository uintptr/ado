use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use crate::{
    config::file::{ConfigFile, OpenAiConfig},
    error::{Error, Result},
    functions::{config::ConfigFunctions, function_handler::FunctionHandler},
    ui::{UiTrait, ux::Console},
};

use log::{error, info};
use reqwest::Client;
use spinner::SpinnerBuilder;

const FUNC_PROMPT_PRE: &str = r#"Dont forget that you have access series of
tools and functions to call to give the user the best possible answer. Here's
the list of functions"#;

use super::{request::OpenAIFunctionRequest, response::OpenAIFunctionResponse};

pub struct OpenAI<'a> {
    client: Client,
    functions: ConfigFunctions,
    openai: &'a OpenAiConfig,
    handler: FunctionHandler<'a>,
    console: Console,
}

impl<'a> OpenAI<'a> {
    pub fn new(config: &'a ConfigFile) -> Result<OpenAI<'a>> {
        let functions = ConfigFunctions::load()?;

        let openai = config.openai()?;

        if openai.key.is_empty() {
            return Err(Error::ApiKeyNotFound);
        }

        Ok(OpenAI {
            client: Client::new(),
            functions,
            openai,
            handler: FunctionHandler::new(config)?,
            console: Console::new()?,
        })
    }

    fn write_to_tmp(&self, file_name: &str, input: &str) -> Result<()> {
        let file_path = Path::new("/tmp").join(file_name);

        let mut f = fs::OpenOptions::new().write(true).truncate(true).create(true).open(file_path)?;

        f.write_all(input.as_bytes())?;

        Ok(())
    }

    pub async fn post_contents(&self, request: &OpenAIFunctionRequest<'_>) -> Result<OpenAIFunctionResponse> {
        let post_data = request.to_json()?;

        self.write_to_tmp("openai_request.json", &post_data)?;

        let res = self
            .client
            .post(&self.openai.url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.openai.key))
            .body(post_data)
            .send()
            .await?;

        let log_msg = format!(
            "post -> code={} reason={}",
            res.status().as_u16(),
            res.status().as_str()
        );

        match res.status().is_success() {
            true => info!("{log_msg}"),
            false => error!("{log_msg}"),
        }

        let response_json = res.text().await?;

        self.write_to_tmp("openai_response.json", &response_json)?;

        let res = OpenAIFunctionResponse::from_string(&response_json)?;

        Ok(res)
    }

    fn build_functions_prompt(&self) -> String {
        let mut func_names: Vec<&str> = Vec::new();

        for f in &self.functions.list {
            func_names.push(&f.name);
        }

        let func_names_str = func_names.join(",");

        format!("{}: {}", FUNC_PROMPT_PRE, func_names_str)
    }

    async fn query_loop(&mut self, query: Option<String>) -> Result<()> {
        let mut req = OpenAIFunctionRequest::new(&self.openai.model, &self.functions);

        let query = match query {
            Some(v) => v,
            None => self.console.read_input()?,
        };

        if let Some(prompt) = &self.openai.prompt {
            req.with_input_role("user", prompt);
        }

        //
        // also tell the AI to make sure of functions
        //
        let functions_prompt = self.build_functions_prompt();
        info!("functions prompt: {functions_prompt}");
        req.with_input_role("user", functions_prompt);
        req.with_input_role("user", query);

        loop {
            let spinner = SpinnerBuilder::new("".into()).start();
            let res = self.post_contents(&req).await?;
            spinner.close();
            print!("\r ");
            io::stdout().flush().unwrap();

            let inputs = res.process_output(&self.console, &self.handler).await?;

            if inputs.is_empty() {
                let query = match self.console.read_input() {
                    Ok(v) => v,
                    Err(e) => break Err(e),
                };

                req.with_input_role("user", query);
            } else {
                req.with_inputs(inputs);
            }
        }
    }

    pub async fn ask(&mut self, query: Option<String>) -> Result<()> {
        let mut local_query = query;

        loop {
            let ret = self.query_loop(local_query).await;

            local_query = None;

            if let Err(Error::ResetInput) = ret {
                continue;
            }

            break ret;
        }
    }
}
