use std::collections::HashMap;

use derive_more::Debug;
use log::{error, info};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    error::{Error, Result},
    tools::assets::{FunctionAssets, FunctionAssetsPlatform},
};

const PARAM_VALID_TYPES: &[&str] = &["object", "string", "integer", "boolean", "array", "number"];

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolProperties {
    #[serde(rename = "type", deserialize_with = "validate_param_type")]
    pub t: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<ToolParameters>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolParameters {
    #[serde(rename = "type", deserialize_with = "validate_param_type")]
    pub t: String,
    pub properties: HashMap<String, ToolProperties>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolFunction {
    #[serde(rename = "type")]
    function_type: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<ToolParameters>,
    returns: Option<ToolParameters>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tools {
    pub list: Vec<ToolFunction>,
}

fn validate_param_type<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let param_type: String = Deserialize::deserialize(deserializer)?;

    match PARAM_VALID_TYPES.contains(&param_type.as_str()) {
        true => Ok(param_type),
        false => Err(serde::de::Error::custom(Error::InvalidInputType { input: param_type })),
    }
}

impl Tools {
    pub fn load() -> Result<Self> {
        let mut list = Vec::new();

        info!("loading function assets");

        for name in FunctionAssets::iter() {
            info!("loading {name}");

            let f = match FunctionAssets::get(&name) {
                Some(v) => v,
                None => {
                    error!("unable to read {name}");
                    continue;
                }
            };

            let content = String::from_utf8_lossy(&f.data);

            let inner_list: Vec<ToolFunction> = serde_json::from_str(&content)?;

            list.extend(inner_list);
        }

        for name in FunctionAssetsPlatform::iter() {
            info!("loading {name}");

            let f = match FunctionAssetsPlatform::get(&name) {
                Some(v) => v,
                None => {
                    error!("unable to read {name}");
                    continue;
                }
            };

            let content = String::from_utf8_lossy(&f.data);

            let inner_list: Vec<ToolFunction> = serde_json::from_str(&content)?;

            list.extend(inner_list);
        }

        //
        // load platform specific functions (wasm vs console)
        //
        info!("function count: {}", list.len());

        Ok(Self { list })
    }
}
