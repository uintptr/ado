use crate::{
    config::loader::AdoConfig,
    data::types::AdoData,
    error::{Error, Result},
    llm::config::OpenAiConfig,
    tools::handler::ToolHandler,
};

use log::{error, info};
use reqwest::Client;

use super::{openai_request::OpenAIRequest, openai_response::OpenAIResponse};

pub struct OpenAIAPI {
    client: Client,
    pub config: OpenAiConfig,
    handler: ToolHandler,
}

impl OpenAIAPI {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        let openai_config = config.openai()?;

        if openai_config.key.is_empty() {
            return Err(Error::ApiKeyNotFound);
        }

        Ok(Self {
            client: Client::new(),
            config: openai_config.clone(),
            handler: ToolHandler::new(config)?,
        })
    }

    pub async fn post_contents(&self, request: &OpenAIRequest) -> Result<OpenAIResponse> {
        let post_data = request.to_json()?;

        let res = self
            .client
            .post(&self.config.url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.config.key))
            .body(post_data)
            .send()
            .await?;

        let log_msg = format!(
            "post {} -> code={} reason={}",
            self.config.url,
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
        let mut req = OpenAIRequest::new(&self.config.model);

        req.with_input_role("user", content);

        let res = self.post_contents(&req).await?;
        let msg = res.process_output(&self.handler).await?.message;
        let output_string: String = msg.try_into()?;

        Ok(output_string)
    }

    pub fn model(&self) -> &str {
        &self.config.model
    }
}
