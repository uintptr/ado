use std::collections::HashMap;

use base64::{Engine, prelude::BASE64_STANDARD};

use reqwest::Response;
use serde::{Serialize, Serializer};

use crate::error::{Error, Result};

pub fn base64_serializer<S>(bytes: &Vec<u8>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&BASE64_STANDARD.encode(bytes))
}

#[derive(Debug, Serialize)]
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
}

#[derive(Debug)]
pub enum AdoData {
    Empty,
    Reset,
    String(String),
    Bytes(Vec<u8>),
    Json(String),
    Base64(String),
    Http(HttpResponse),
    SearchData(String),
    UsageString(String),
}

impl TryFrom<AdoData> for String {
    type Error = Error;

    fn try_from(value: AdoData) -> Result<String> {
        let s = match value {
            AdoData::Empty => "".into(),
            AdoData::Reset => "".into(),
            AdoData::String(s) => s,
            AdoData::Json(s) => s,
            AdoData::Base64(s) => s,
            AdoData::Bytes(b) => BASE64_STANDARD.encode(b),
            AdoData::Http(h) => serde_json::to_string(&h)?,
            AdoData::SearchData(s) => s,
            AdoData::UsageString(s) => s,
        };

        Ok(s)
    }
}

impl AdoData {
    pub fn to_base64(&self) -> Result<String> {
        let out = match self {
            AdoData::Empty => BASE64_STANDARD.encode("".to_string()),
            AdoData::Reset => BASE64_STANDARD.encode("".to_string()),
            AdoData::String(s) => BASE64_STANDARD.encode(s),
            AdoData::Json(s) => BASE64_STANDARD.encode(s),
            AdoData::Base64(s) => BASE64_STANDARD.encode(s),
            AdoData::Bytes(b) => BASE64_STANDARD.encode(b),
            AdoData::Http(h) => {
                let json_str = serde_json::to_string(h)?;
                BASE64_STANDARD.encode(json_str)
            }
            AdoData::SearchData(s) => BASE64_STANDARD.encode(s),
            AdoData::UsageString(s) => BASE64_STANDARD.encode(s),
        };

        Ok(out)
    }
}
