use std::{collections::HashMap, fs, path::Path};

use derive_more::Debug;
use log::info;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    error::{Error, Result},
    staples::find_file,
};

const FUNCTIONS_FILE: &str = "functions.json";

const PARAM_VALID_TYPES: &[&str] = &["object", "string", "integer", "boolean", "array"];

#[derive(Debug, Serialize, Deserialize)]
pub struct Properties {
    #[serde(rename = "type", deserialize_with = "validate_param_type")]
    t: String,
    description: String,
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
    parameters: Parameters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFunctions {
    #[debug(skip)]
    json_data: String,
    pub list: Vec<ConfigFunction>,
}

fn validate_param_type<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let param_type: String = Deserialize::deserialize(deserializer)?;

    match PARAM_VALID_TYPES.contains(&param_type.as_str()) {
        true => Ok(param_type),
        false => {
            return Err(serde::de::Error::custom(Error::InvalidInputType {
                input: param_type,
            }));
        }
    }
}

impl ConfigFunctions {
    pub fn load() -> Result<Self> {
        let config_rel_path = Path::new("config").join(FUNCTIONS_FILE);

        let config_file = find_file(config_rel_path)?;

        let json_data = fs::read_to_string(config_file)?;

        let list: Vec<ConfigFunction> = serde_json::from_str(&json_data)?;

        for f in list.iter() {
            info!("function: {}", f.name)
        }

        Ok(ConfigFunctions { json_data, list })
    }
}
