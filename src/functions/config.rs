use std::{collections::HashMap, fs, path::Path};

use derive_more::Debug;
use log::{error, info};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    error::{Error, Result},
    staples::find_file,
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
        let config_dir_rel = Path::new("config").join("functions");
        let config_dir = find_file(config_dir_rel)?;

        let json_patt = format!("{}/*.json", config_dir.display());

        let glob_files = glob::glob(&json_patt)?;

        let mut list = Vec::new();

        for file_res in glob_files {
            let file = match file_res {
                Ok(v) => v,
                Err(e) => {
                    error!("{e}");
                    continue;
                }
            };

            let json_data = fs::read_to_string(file)?;
            let inner_list: Vec<ConfigFunction> = serde_json::from_str(&json_data)?;

            list.extend(inner_list);
        }

        info!("function count: {}", list.len());

        /*
        let json_str = serde_json::to_string_pretty(&list)?;

        println!("{json_str}");

        if json_str.len() > 0 {
            panic!("ok");
        }
        */

        Ok(ConfigFunctions { list })
    }
}
