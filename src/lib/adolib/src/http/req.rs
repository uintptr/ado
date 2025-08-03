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
