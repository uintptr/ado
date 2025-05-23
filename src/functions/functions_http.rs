use std::collections::HashMap;

use base64::{Engine, prelude::BASE64_STANDARD};
use log::info;
use minreq::Response;
use serde::{Serialize, Serializer};

use crate::error::Result;

use super::function_args::FunctionArgs;

#[derive(Debug, Serialize)]
pub struct HttpResponse<'a> {
    url: &'a str,
    code: i32,
    reason: &'a str,
    headers: &'a HashMap<String, String>,
    #[serde(serialize_with = "base64_serializer")]
    base64_data: &'a [u8],
}

pub fn base64_serializer<S>(bytes: &[u8], serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&BASE64_STANDARD.encode(bytes))
}

impl<'a> From<&'a Response> for HttpResponse<'a> {
    fn from(response: &'a Response) -> Self {
        Self {
            url: &response.url,
            code: response.status_code,
            reason: &response.reason_phrase,
            headers: &response.headers,
            base64_data: response.as_bytes(),
        }
    }
}

#[derive(Default)]
pub struct FunctionsHttp;

impl FunctionsHttp {
    pub fn new() -> FunctionsHttp {
        FunctionsHttp {}
    }

    pub fn http_get(&self, url: &str, headers: Option<HashMap<&str, &str>>) -> Result<String> {
        let mut req = minreq::get(url);

        if let Some(h) = headers {
            req = req.with_headers(h);
        }

        let res = req.send()?;

        let res = HttpResponse::from(&res);

        info!("{} -> {} len={}", res.url, res.code, res.base64_data.len());

        //
        // we send back a json string
        //
        let res_json = serde_json::to_string_pretty(&res)?;
        Ok(res_json)
    }

    pub fn get(&self, args: &FunctionArgs) -> Result<String> {
        let url = args.get_string("url")?;

        //
        // optional
        //
        let list = args.get_kv_list("http_headers").ok();
        let headers = list.as_ref().map(|v| args.kv_list_to_map(v));

        self.http_get(url, headers)
    }
}

#[cfg(test)]
mod tests {
    use crate::staples::setup_logger;

    use super::FunctionsHttp;

    use log::info;

    #[test]
    fn test_get() {
        setup_logger(true).unwrap();

        let http = FunctionsHttp::new();

        let json_data = http.http_get("http://localhost:8000", None).unwrap();

        info!("{json_data}");
    }
}
