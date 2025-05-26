use std::{fs, io::Write, path::Path};

use crate::{
    config::file::{ConfigFile, OpenAiConfig},
    error::{Error, Result},
    functions::{config::ConfigFunctions, function_handler::FunctionHandler},
    ui::{UiTrait, ui::Console},
};

use log::{error, info};

use super::{request::OpenAIFunctionRequest, response::OpenAIFunctionResponse};

pub struct OpenAI {
    functions: ConfigFunctions,
    openai: OpenAiConfig,
    handler: FunctionHandler,
    console: Console,
}

impl OpenAI {
    pub fn new() -> Result<Self> {
        let functions = ConfigFunctions::load()?;
        let config = ConfigFile::load()?;

        let openai = config.openai()?;

        if openai.key.is_empty() {
            return Err(Error::ApiKeyNotFound);
        }

        Ok(OpenAI {
            functions,
            openai,
            handler: FunctionHandler::new()?,
            console: Console::new()?,
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

        let res = minreq::post(&self.openai.url)
            .with_header("Content-Type", "application/json")
            .with_header("Authorization", format!("Bearer {}", self.openai.key))
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

    pub fn ask(&mut self, query: Option<String>) -> Result<()> {
        let mut req = OpenAIFunctionRequest::new(&self.openai.model, &self.functions);

        let query = match query {
            Some(v) => v,
            None => self.console.readline()?,
        };

        if let Some(prompt) = &self.openai.prompt {
            req.with_input_role("user", prompt);
        }

        req.with_input_role("user", query.as_ref());

        loop {
            let res = self.post_contents(&req)?;

            let inputs = res.process_output(&self.console, &self.handler)?;

            if inputs.is_empty() {
                let query = match self.console.readline() {
                    Ok(v) => v,
                    Err(e) => {
                        error!("{e}");
                        break;
                    }
                };

                req.with_input_role("user", query.as_ref());
            } else {
                req.with_inputs(inputs);
            }
        }

        Ok(())
    }
}
