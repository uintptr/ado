use crate::{
    config::loader::{ConfigFile, OpenAiConfig},
    data::AdoData,
    error::{Error, Result},
    tools::handler::FunctionHandler,
};

use log::{error, info};
use reqwest::Client;

use super::{request::OpenAIRequest, response::OpenAIResponse};

pub struct LLM {
    client: Client,
    openai: OpenAiConfig,
    handler: FunctionHandler,
}

impl LLM {
    pub fn new(config: &ConfigFile) -> Result<LLM> {
        let openai = config.openai()?;

        if openai.key.is_empty() {
            return Err(Error::ApiKeyNotFound);
        }

        Ok(LLM {
            client: Client::new(),
            openai: openai.clone(),
            handler: FunctionHandler::new(config)?,
        })
    }

    pub async fn post_contents(&self, request: &OpenAIRequest) -> Result<OpenAIResponse> {
        let post_data = request.to_json()?;

        let res = self
            .client
            .post(&self.openai.url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.openai.key))
            .body(post_data)
            .send()
            .await?;

        let log_msg = format!(
            "post {} -> code={} reason={}",
            self.openai.url,
            res.status().as_u16(),
            res.status().as_str()
        );

        for (k, v) in res.headers() {
            info!("{k}:{v:?}");
        }

        match res.status().is_success() {
            true => info!("{log_msg}"),
            false => error!("{log_msg}"),
        }

        let response_json = res.text().await?;

        let res = OpenAIResponse::from_string(&response_json)?;

        Ok(res)
    }

    pub async fn query(&mut self, req: &mut OpenAIRequest) -> Result<AdoData> {
        loop {
            let res = self.post_contents(req).await?;

            let outputs = res.process_output(&self.handler).await?;

            if outputs.inputs.is_empty() {
                //
                // Nothing to do
                //
                break Ok(outputs.message);
            }

            req.with_inputs(outputs.inputs);
        }
    }

    pub async fn message<S>(&self, content: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        let mut req = OpenAIRequest::new(&self.openai.model);

        req.with_input_role("user", content);

        let res = self.post_contents(&req).await?;
        let msg = res.process_output(&self.handler).await?.message;
        let output_string: String = msg.try_into()?;

        Ok(output_string)
    }
}
