use std::{fs, io::Write, path::Path};

use crate::{
    config::{AdoConfig, OpenAiConfig},
    error::Result,
    functions::{config::ConfigFunctions, function_handler::FunctionHandler},
};

use log::{error, info};

use super::{request::OpenAIFunctionRequest, response::OpenAIFunctionResponse};

pub struct OpenAI {
    functions: ConfigFunctions,
    config: OpenAiConfig,
    handler: FunctionHandler,
}

impl OpenAI {
    pub fn new() -> Result<Self> {
        let functions = ConfigFunctions::load()?;
        let config = AdoConfig::load()?;

        let config = config.openai()?;

        let handler = FunctionHandler::new()?;

        Ok(OpenAI {
            functions,
            config,
            handler,
        })
    }

    fn write_to_tmp(&self, file_name: &str, input: &str) -> Result<()> {
        let file_path = Path::new("/tmp").join(file_name);

        let mut f = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file_path)?;

        f.write_all(input.as_bytes())?;

        Ok(())
    }

    pub fn post_contents(&self, request: &OpenAIFunctionRequest) -> Result<OpenAIFunctionResponse> {
        let post_data = request.to_json()?;

        self.write_to_tmp("openai_request.json", &post_data)?;

        let res = minreq::post(&self.config.url)
            .with_header("Content-Type", "application/json")
            .with_header("Authorization", format!("Bearer {}", self.config.key))
            .with_body(post_data)
            .send()?;

        let log_msg = format!(
            "post -> code={} reason={}",
            res.status_code, res.reason_phrase
        );

        match res.status_code {
            200..299 => info!("{log_msg}"),
            _ => error!("{log_msg}"),
        }

        let response_json = res.as_str()?;

        self.write_to_tmp("openai_response.json", response_json)?;

        let res = OpenAIFunctionResponse::from_string(response_json)?;

        Ok(res)
    }

    pub fn ask<S>(&self, query: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        let mut req = OpenAIFunctionRequest::new(&self.config.model, &self.functions);

        req.with_input_role("user", query.as_ref());

        let res = self.post_contents(&req)?;

        let inputs = res.call_functions(&self.handler)?;

        req.with_inputs(inputs);

        //
        // with the updated request
        //
        let res = self.post_contents(&req)?;

        res.content_text()
    }
}
