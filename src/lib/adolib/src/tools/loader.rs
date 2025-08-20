use std::{collections::HashMap, str::FromStr};

use derive_more::Debug;
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    tools::assets::{FunctionAssets, FunctionAssetsPlatform},
};

#[derive(Debug, Deserialize, Serialize)]
pub enum ToolType {
    #[serde(rename = "object")]
    Object,
    #[serde(rename = "string")]
    String,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "array")]
    Array,
    #[serde(rename = "number")]
    Number,
}

impl FromStr for ToolType {
    type Err = Error;

    fn from_str(s: &str) -> Result<ToolType> {
        match s {
            "object" => Ok(ToolType::Object),
            "string" => Ok(ToolType::String),
            "integer" => Ok(ToolType::Integer),
            "boolean" => Ok(ToolType::Boolean),
            "array" => Ok(ToolType::Array),
            "number" => Ok(ToolType::Number),
            _ => Err(Error::NotImplemented),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolProperties {
    #[serde(rename = "type")]
    pub property_type: ToolType,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<ToolParameters>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolParameters {
    #[serde(rename = "type")]
    pub param_type: ToolType,
    pub properties: HashMap<String, ToolProperties>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolFunction {
    #[serde(rename = "type")]
    function_type: ToolType,
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
