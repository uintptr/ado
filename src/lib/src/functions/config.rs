use std::collections::HashMap;

use derive_more::Debug;
use log::{error, info};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    error::{Error, Result},
    functions::assets::Assets,
};

const PARAM_VALID_TYPES: &[&str] = &["object", "string", "integer", "boolean", "array"];

#[derive(Debug, Serialize, Deserialize)]
pub struct Properties {
    #[serde(rename = "type", deserialize_with = "validate_param_type")]
    t: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<Parameters>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameters {
    #[serde(rename = "type", deserialize_with = "validate_param_type")]
    t: String,
    properties: HashMap<String, Properties>,
    required: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConfigFunctionCall {
    pub name: String,
    pub args: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFunction {
    #[serde(rename = "type")]
    t: String,
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<Parameters>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFunctions {
    pub list: Vec<ConfigFunction>,
}

fn validate_param_type<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let param_type: String = Deserialize::deserialize(deserializer)?;

    match PARAM_VALID_TYPES.contains(&param_type.as_str()) {
        true => Ok(param_type),
        false => Err(serde::de::Error::custom(Error::InvalidInputType {
            input: param_type,
        })),
    }
}

impl ConfigFunctions {
    pub fn load() -> Result<Self> {
        let mut list = Vec::new();

        for name in Assets::iter() {
            let f = match Assets::get(&name) {
                Some(v) => v,
                None => {
                    error!("unable to read {}", name);
                    continue;
                }
            };

            let content = String::from_utf8_lossy(&f.data);

            let inner_list: Vec<ConfigFunction> = serde_json::from_str(&content)?;

            list.extend(inner_list);
        }

        info!("function count: {}", list.len());

        Ok(ConfigFunctions { list })
    }
}
