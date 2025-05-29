use std::collections::HashMap;

use base64::{Engine, prelude::BASE64_STANDARD};
use reqwest::Client;
use serde::{Serialize, Serializer};
use tokio::fs;

use crate::error::Result;
use log::info;

use super::function_args::FunctionArgs;

#[derive(Debug, Serialize)]
pub struct HttpResponse<'a> {
    url: &'a str,
    code: u16,
    headers: HashMap<String, String>,
    #[serde(serialize_with = "base64_serializer")]
    base64_data: &'a [u8],
}

pub fn base64_serializer<S>(bytes: &[u8], serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&BASE64_STANDARD.encode(bytes))
}

#[derive(Default)]
pub struct FunctionsHttp {
    client: Client,
}

impl FunctionsHttp {
    pub fn new() -> FunctionsHttp {
        FunctionsHttp {
            client: Client::new(),
        }
    }

    pub async fn http_get(
        &self,
        url: &str,
        headers: Option<HashMap<&str, &str>>,
    ) -> Result<String> {
        let mut req = self.client.get(url);

        if let Some(h) = headers {
            for (k, v) in h {
                req = req.header(k, v)
            }
        }

        let res = req.send().await?;

        let status_code = res.status().as_u16();

        let mut headers = HashMap::new();
        for (k, ov) in res.headers().iter() {
            if let Ok(v) = ov.to_str() {
                headers.insert(k.as_str().to_string(), v.to_string());
            }
        }

        let data = res.bytes().await?;

        let local_res = HttpResponse {
            url: url,
            code: status_code,
            headers: headers,
            base64_data: &data,
        };

        info!(
            "{} -> {} len={}",
            local_res.url,
            local_res.code,
            local_res.base64_data.len()
        );

        //
        // we send back a json string
        //
        let res_json = serde_json::to_string_pretty(&local_res)?;

        fs::write("/tmp/bleh.json", &res_json).await?;

        Ok(res_json)
    }

    pub async fn get(&self, args: &FunctionArgs) -> Result<String> {
        let url = args.get_string("url")?;

        //
        // optional
        //
        let list = args.get_kv_list("http_headers").ok();
        let headers = list.as_ref().map(|v| args.kv_list_to_map(v));

        self.http_get(url, headers).await
    }
}

#[cfg(test)]
mod tests {
    use crate::staples::setup_logger;

    use super::FunctionsHttp;

    use log::info;

    #[tokio::test]
    async fn test_get() {
        setup_logger(true).unwrap();

        let http = FunctionsHttp::new();

        let json_data = http.http_get("http://localhost:8000", None).await.unwrap();

        info!("{json_data}");
    }
}
