use std::{collections::HashMap, io::Read, process::Child, time::Instant};

use base64::{Engine, prelude::BASE64_STANDARD};

use reqwest::Response;
use serde::{Deserialize, Serialize, Serializer};

use crate::{
    error::{Error, Result},
    ui::commands::StatusInfo,
};

pub fn base64_serializer<S>(bytes: &Vec<u8>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&BASE64_STANDARD.encode(bytes))
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ShellExit {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
    pub execution_time: u64,
}

impl ShellExit {
    pub fn from_child(mut child: Child) -> Result<ShellExit> {
        let start = Instant::now();

        let exit = child.wait()?;

        let duration = start.elapsed();

        let stdout = match child.stdout.as_mut() {
            Some(v) => {
                let mut buf = Vec::new();
                v.read_to_end(&mut buf)?;
                buf
            }
            None => vec![],
        };

        let stdout = String::from_utf8(stdout).unwrap_or("invalid stdout data".to_string());

        let stderr = match child.stderr.as_mut() {
            Some(v) => {
                let mut buf = Vec::new();
                v.read_to_end(&mut buf)?;
                buf
            }
            None => vec![],
        };

        let stderr = String::from_utf8(stderr).unwrap_or("invalid stderr data".to_string());

        let exit_code = exit.code().unwrap_or(1);

        Ok(ShellExit {
            stdout,
            stderr,
            exit_code,
            timed_out: false,
            execution_time: duration.as_secs(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
    Shell(ShellExit),
    Status(StatusInfo),
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
            AdoData::Shell(e) => serde_json::to_string(&e)?,
            AdoData::Status(s) => serde_json::to_string(&s)?,
        };

        Ok(s)
    }
}

impl TryFrom<AdoData> for Vec<u8> {
    type Error = Error;

    fn try_from(value: AdoData) -> Result<Vec<u8>> {
        let s = match value {
            AdoData::Empty => vec![],
            AdoData::Reset => vec![],
            AdoData::String(s) => s.into_bytes(),
            AdoData::Json(s) => s.into_bytes(),
            AdoData::Base64(s) => s.into_bytes(),
            AdoData::Bytes(b) => b,
            AdoData::Http(h) => serde_json::to_vec(&h)?,
            AdoData::SearchData(s) => s.into_bytes(),
            AdoData::UsageString(s) => s.into_bytes(),
            AdoData::Shell(e) => serde_json::to_vec(&e)?,
            AdoData::Status(s) => serde_json::to_vec(&s)?,
        };
        Ok(s)
    }
}

impl AdoData {
    pub fn to_base64(&self) -> Result<String> {
        let out = match self {
            AdoData::Empty => BASE64_STANDARD.encode(""),
            AdoData::Reset => BASE64_STANDARD.encode(""),
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
            AdoData::Shell(e) => {
                let json_str = serde_json::to_string(e)?;
                BASE64_STANDARD.encode(json_str)
            }
            AdoData::Status(s) => {
                let json_str = serde_json::to_string(s)?;
                BASE64_STANDARD.encode(json_str)
            }
        };

        Ok(out)
    }
}
