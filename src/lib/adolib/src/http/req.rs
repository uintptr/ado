use std::collections::HashMap;

use reqwest::{Client, Response};

use crate::error::Result;
use log::info;

use serde::{Deserialize, Serialize};

use crate::data::types::AdoDataMarkdown;

use crate::data::base64_serializer::base64_serializer;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpResponse {
    pub url: String,
    pub code: u16,
    pub headers: HashMap<String, String>,
    #[serde(serialize_with = "base64_serializer")]
    pub data: Vec<u8>,
}

impl HttpResponse {
    pub async fn from_response(res: Response) -> Result<HttpResponse> {
        let mut headers = HashMap::new();
        for (k, ov) in res.headers().iter() {
            if let Ok(v) = ov.to_str() {
                headers.insert(k.as_str().to_string(), v.to_string());
            }
        }

        let url = res.url().to_string();
        let code = res.status().as_u16();

        let data = res.bytes().await?;

        let local_res = HttpResponse {
            url,
            code,
            headers,
            data: data.to_vec(),
        };

        Ok(local_res)
    }

    pub fn is_success(&self) -> bool {
        matches!(self.code, 200..299)
    }
}

impl AdoDataMarkdown for HttpResponse {
    fn to_markdown(self) -> Result<String> {
        let mut lines = Vec::new();
        lines.push("# Http Results".to_string());
        lines.push(format!(" * url: {}", self.url));
        lines.push(format!(" * code: {}", self.code));

        Ok(lines.join("\n"))
    }
}

#[derive(Debug, Clone)]
pub struct Http {
    client: Client,
}

fn log_response(res: &Response) {
    let content_len = res.content_length().unwrap_or(0);
    info!("{} -> {} len={}", res.url(), res.status().as_u16(), content_len);
}

impl Default for Http {
    fn default() -> Self {
        Self::new()
    }
}

impl Http {
    pub fn new() -> Http {
        Http { client: Client::new() }
    }

    pub async fn put<S, D>(&self, url: S, headers: Option<HashMap<&str, &str>>, data: D) -> Result<HttpResponse>
    where
        S: AsRef<str>,
        D: AsRef<[u8]>,
    {
        let mut req = self.client.put(url.as_ref());

        if let Some(h) = headers {
            for (k, v) in h {
                req = req.header(k, v)
            }
        }

        req = req.body(data.as_ref().to_vec());

        let res = req.send().await?;

        log_response(&res);

        HttpResponse::from_response(res).await
    }

    pub async fn get<S>(&self, url: S, headers: Option<HashMap<&str, &str>>) -> Result<HttpResponse>
    where
        S: AsRef<str>,
    {
        let mut req = self.client.get(url.as_ref());

        if let Some(h) = headers {
            for (k, v) in h {
                req = req.header(k, v)
            }
        }

        let res = req.send().await?;

        log_response(&res);

        HttpResponse::from_response(res).await
    }

    pub async fn post(&self, url: &str, headers: Option<HashMap<&str, &str>>) -> Result<HttpResponse> {
        let mut req = self.client.post(url);

        if let Some(h) = headers {
            for (k, v) in h {
                req = req.header(k, v)
            }
        }

        let res = req.send().await?;

        log_response(&res);

        HttpResponse::from_response(res).await
    }
}
