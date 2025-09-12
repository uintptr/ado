use std::str::FromStr;

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose};
use log::info;
use omcp::{client::types::BakedMcpToolTrait, types::McpParams};
use reqwest::{
    Client,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use serde::Serialize;
use serde_json::Map;

use crate::error::{Error, Result};

#[derive(Serialize)]
pub struct ToolHttpResult {
    b64_response: String,
    http_status: u16,
}

pub struct ToolHttpGet {
    client: Client,
}
pub struct ToolHttpPost {}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////
// HTTP GET
///////////////////////////////////////

impl ToolHttpGet {
    pub fn new() -> Self {
        let client = Client::new();

        Self { client }
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolHttpGet {
    type Error = Error;

    async fn call(&mut self, params: &McpParams) -> Result<String> {
        let url = params.get_string("url")?;

        let headers = match params.get_object("http_headers") {
            Ok(v) => v,
            Err(_) => Map::new(),
        };

        info!("url={url} headers={:?}", headers);

        let mut req_headers = HeaderMap::new();

        for (k, v) in headers {
            let key = HeaderName::from_str(&k)?;
            let v = v.as_str().ok_or(Error::InvalidFormat)?;
            let value = HeaderValue::from_str(v)?;
            req_headers.insert(key, value);
        }

        let res = self.client.get(url).headers(req_headers).send().await?;

        let http_status = res.status().as_u16();

        let data = res.bytes().await?;

        let b64_response = general_purpose::STANDARD.encode(data);

        let result = ToolHttpResult {
            b64_response,
            http_status,
        };
        Ok(serde_json::to_string(&result)?)
    }
}

///////////////////////////////////////
// HTTP GET
///////////////////////////////////////

impl ToolHttpPost {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolHttpPost {
    type Error = Error;

    async fn call(&mut self, _params: &McpParams) -> Result<String> {
        unimplemented!()
    }
}
