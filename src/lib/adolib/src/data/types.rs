use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Deserialize, Serialize};

use crate::{
    data::search::GoogleSearchData,
    error::{Error, Result},
    http::req::HttpResponse,
    shell::ShellExit,
    ui::commands::StatusInfo,
};

pub trait AdoDataMarkdown {
    fn to_markdown(self) -> Result<String>;
}

pub trait AdoDataBase64 {
    fn to_base64(self) -> Result<String>;
}

pub trait AdoDataDisplay: AdoDataMarkdown + AdoDataBase64 {}

#[derive(Debug, Serialize, Deserialize)]
pub enum AdoData {
    Empty,
    Reset,
    String(String),
    Bytes(Vec<u8>),
    Json(String),
    Base64(String),
    Http(HttpResponse),
    SearchData(GoogleSearchData),
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
            AdoData::SearchData(d) => d.to_markdown()?,
            AdoData::Shell(s) => s.to_markdown()?,
            d => d.to_markdown()?,
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
