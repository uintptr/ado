use std::{path::PathBuf, str::FromStr};

use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdoDataArtifactType {
    Code,
    Diff,
    File,
    Command,
    Note,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdoDataStatus {
    Ok,
    Error,
    Partial,
}

#[derive(Serialize, Deserialize)]
pub struct AdoDataMeta {
    pub status: AdoDataStatus,
    pub intent: String,
    pub confidence: f32,
}

#[derive(Serialize, Deserialize)]
pub struct AdoDataArtifact {
    #[serde(rename = "type")]
    pub artifact_type: AdoDataArtifactType,
    pub language: Option<String>,
    pub path: Option<PathBuf>,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct AdoDataResponse {
    pub message: String,
    pub artifacts: Option<Vec<AdoDataArtifact>>,
}

#[derive(Serialize, Deserialize)]
pub struct AdoDataError {
    pub code: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct AdoData {
    pub meta: AdoDataMeta,
    pub response: AdoDataResponse,
    pub error: Option<AdoDataError>,
}

impl FromStr for AdoData {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("```json\n").unwrap_or(s);
        let s = s.strip_suffix("\n```").unwrap_or(s);

        let data: AdoData = match serde_json::from_str(s) {
            Ok(v) => v,
            Err(e) => {
                error!("Deserialization failure ({e})\n{s}\n");
                return Err(e.into());
            }
        };

        Ok(data)
    }
}

impl ToString for AdoData {
    fn to_string(&self) -> String {
        let str = serde_json::to_string_pretty(self).unwrap_or_default();
        str
    }
}
