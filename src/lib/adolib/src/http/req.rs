use std::collections::HashMap;

use reqwest::{Client, Response};

use crate::{data::HttpResponse, error::Result};
use log::info;

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

    pub async fn get(&self, url: &str, headers: Option<HashMap<&str, &str>>) -> Result<HttpResponse> {
        let mut req = self.client.get(url);

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
