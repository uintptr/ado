use std::{fmt::Display, path::PathBuf, str::FromStr};

use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

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

impl Display for AdoDataStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AdoDataStatus::Ok => "Ok",
            AdoDataStatus::Error => "Error",
            AdoDataStatus::Partial => "Partial",
        };

        write!(f, "{s}")
    }
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

/// JSON Schema describing [`AdoData`], shared by every backend to drive
/// provider-native structured outputs (Claude `output_config.format`, Ollama
/// `format`). The schema enforces the *shape* of the response so the model can
/// no longer emit malformed JSON or wrap it in prose/fences.
#[must_use]
pub fn ado_data_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "meta": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "status": { "type": "string", "enum": ["ok", "error", "partial"] },
                    "intent": { "type": "string" },
                    "confidence": { "type": "number" }
                },
                "required": ["status", "intent", "confidence"]
            },
            "response": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "message": { "type": "string" },
                    "artifacts": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "additionalProperties": false,
                            "properties": {
                                "type": { "type": "string", "enum": ["code", "diff", "file", "command", "note"] },
                                "language": { "type": ["string", "null"] },
                                "path": { "type": ["string", "null"] },
                                "content": { "type": "string" }
                            },
                            "required": ["type", "language", "path", "content"]
                        }
                    }
                },
                "required": ["message", "artifacts"]
            },
            "error": {
                "type": ["object", "null"],
                "additionalProperties": false,
                "properties": {
                    "code": { "type": "string" },
                    "message": { "type": "string" }
                },
                "required": ["code", "message"]
            }
        },
        "required": ["meta", "response", "error"]
    })
}

impl FromStr for AdoData {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("```json\n").unwrap_or(s);
        let s = s.strip_suffix("\n```").unwrap_or(s);

        if let Ok(v) = serde_json::from_str::<Value>(s)
            && let Ok(pretty_s) = serde_json::to_string_pretty(&v)
        {
            info!("\n{pretty_s}");
        }

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

impl Display for AdoData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string_pretty(self).unwrap_or_default();
        write!(f, "{s}")
    }
}

impl Display for AdoDataArtifactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AdoDataArtifactType::Code => "code",
            AdoDataArtifactType::Command => "command",
            AdoDataArtifactType::Diff => "diff",
            AdoDataArtifactType::File => "file",
            AdoDataArtifactType::Note => "note",
        };

        write!(f, "{s}")
    }
}
