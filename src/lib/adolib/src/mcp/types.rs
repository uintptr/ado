use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    tools::loader::{ToolFunction, ToolParameters, ToolProperties},
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
    #[serde(rename = "function")]
    Function,
}

#[derive(Debug, Serialize)]
pub struct McpToolProperty {
    #[serde(rename = "type")]
    property_type: ToolType,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<McpToolSchema>,
}

#[derive(Debug, Serialize)]
pub struct McpToolSchema {
    #[serde(rename = "type")]
    schema_type: ToolType,
    properties: HashMap<String, McpToolProperty>,
    required: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct McpTool {
    name: String,
    description: String,
    input_schema: Option<McpToolSchema>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct McpConfig {
    #[serde(rename = "type")]
    config_type: String,
    url: Option<String>,
    authorization_token: Option<String>,
}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////

impl TryFrom<ToolProperties> for McpToolProperty {
    type Error = Error;

    fn try_from(props: ToolProperties) -> Result<Self> {
        let items = match props.items {
            Some(v) => {
                let claude_items: McpToolSchema = v.try_into()?;
                Some(claude_items)
            }
            None => None,
        };

        let claude_prop = Self {
            property_type: props.property_type,
            description: props.description,
            items,
        };

        Ok(claude_prop)
    }
}

impl TryFrom<ToolParameters> for McpToolSchema {
    type Error = Error;

    fn try_from(params: ToolParameters) -> Result<Self> {
        let mut properties: HashMap<String, McpToolProperty> = HashMap::new();

        for (k, v) in params.properties {
            let prop: McpToolProperty = v.try_into()?;
            properties.insert(k, prop);
        }

        let mut required: Vec<String> = Vec::new();

        if let Some(req_list) = params.required {
            for param in req_list {
                required.push(param);
            }
        }

        let claude_schema = Self {
            schema_type: params.param_type,
            properties,
            required,
        };

        Ok(claude_schema)
    }
}

impl TryFrom<ToolFunction> for McpTool {
    type Error = Error;

    fn try_from(tool: ToolFunction) -> Result<Self> {
        let input_schema = match tool.parameters {
            Some(v) => {
                let claude_schema: McpToolSchema = v.try_into()?;
                Some(claude_schema)
            }
            None => None,
        };

        let claude_tool = McpTool {
            name: tool.name,
            description: tool.description,
            input_schema,
        };

        Ok(claude_tool)
    }
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
            "function" => Ok(ToolType::Function),
            _ => Err(Error::NotImplemented),
        }
    }
}
