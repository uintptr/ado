use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    http::req::HttpResponse,
    llm::chain::LLMUsage,
    search::google::GoogleSearchResults,
    shell::ShellExit,
    ui::status::StatusInfo,
};

pub trait AdoDataMarkdown {
    fn to_markdown(self) -> Result<String>;
}

pub trait AdoDataBase64 {
    fn to_base64(self) -> Result<String>;
}

pub trait AdoDataDisplay: AdoDataMarkdown + AdoDataBase64 {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AdoData {
    Empty,
    Reset,
    String(String),
    Bytes(Vec<u8>),
    Json(String),
    Base64(String),
    Http(HttpResponse),
    SearchData(GoogleSearchResults),
    LlmUsage(LLMUsage),
    UsageString(String),
    Shell(ShellExit),
    Status(StatusInfo),
}

impl AdoDataMarkdown for String {
    fn to_markdown(self) -> Result<String> {
        Ok(format!("```\n{}\n```", self))
    }
}

impl AdoDataMarkdown for AdoData {
    fn to_markdown(self) -> Result<String> {
        let md = match self {
            AdoData::Empty => "".to_string(),
            AdoData::Reset => "".to_string(),
            AdoData::String(s) => s.to_markdown()?,
            AdoData::Bytes(_) => unimplemented!(),
            AdoData::Json(s) => s.to_markdown()?,
            AdoData::Base64(b) => b.to_markdown()?,
            AdoData::SearchData(d) => d.to_markdown()?,
            AdoData::Http(h) => h.to_markdown()?,
            AdoData::UsageString(s) => s.to_markdown()?,
            AdoData::Shell(s) => s.to_markdown()?,
            AdoData::Status(s) => s.to_markdown()?,
            AdoData::LlmUsage(s) => s.to_markdown()?,
        };

        Ok(md)
    }
}

impl AdoDataBase64 for AdoData {
    fn to_base64(self) -> Result<String> {
        let out = match self {
            AdoData::Empty => BASE64_STANDARD.encode(""),
            AdoData::Reset => BASE64_STANDARD.encode(""),
            AdoData::String(s) => BASE64_STANDARD.encode(s),
            AdoData::Json(s) => BASE64_STANDARD.encode(s),
            AdoData::Base64(s) => BASE64_STANDARD.encode(s),
            AdoData::Bytes(b) => BASE64_STANDARD.encode(b),
            AdoData::Http(h) => {
                let json_str = serde_json::to_string(&h)?;
                BASE64_STANDARD.encode(json_str)
            }
            AdoData::SearchData(s) => {
                let json_str = serde_json::to_string(&s)?;
                BASE64_STANDARD.encode(json_str)
            }
            AdoData::UsageString(s) => BASE64_STANDARD.encode(s),
            AdoData::Shell(e) => {
                let json_str = serde_json::to_string(&e)?;
                BASE64_STANDARD.encode(json_str)
            }
            AdoData::Status(s) => {
                let json_str = serde_json::to_string(&s)?;
                BASE64_STANDARD.encode(json_str)
            }
            AdoData::LlmUsage(u) => {
                let json_str = serde_json::to_string(&u)?;
                BASE64_STANDARD.encode(json_str)
            }
        };

        Ok(out)
    }
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
            AdoData::SearchData(s) => s.json_string,
            AdoData::UsageString(s) => s,
            u => serde_json::to_string(&u)?,
        };

        Ok(s)
    }
}
