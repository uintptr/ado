use std::collections::HashMap;

use serde::Serialize;

use crate::{
    error::{Error, Result},
    tools::loader::{ToolFunction, ToolParameters, ToolProperties, ToolType},
};

#[derive(Debug, Serialize)]
pub struct ClaudeToolProperty {
    #[serde(rename = "type")]
    property_type: ToolType,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<ClaudeToolSchema>,
}

impl TryFrom<ToolProperties> for ClaudeToolProperty {
    type Error = Error;

    fn try_from(props: ToolProperties) -> Result<Self> {
        let items = match props.items {
            Some(v) => {
                let claude_items: ClaudeToolSchema = v.try_into()?;
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

#[derive(Debug, Serialize)]
pub struct ClaudeToolSchema {
    #[serde(rename = "type")]
    schema_type: ToolType,
    properties: HashMap<String, ClaudeToolProperty>,
    required: Vec<String>,
}

impl TryFrom<ToolParameters> for ClaudeToolSchema {
    type Error = Error;

    fn try_from(params: ToolParameters) -> Result<Self> {
        let mut properties: HashMap<String, ClaudeToolProperty> = HashMap::new();

        for (k, v) in params.properties {
            let prop: ClaudeToolProperty = v.try_into()?;
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

#[derive(Debug, Serialize)]
pub struct ClaudeTool {
    name: String,
    description: String,
    input_schema: Option<ClaudeToolSchema>,
}

impl TryFrom<ToolFunction> for ClaudeTool {
    type Error = Error;

    fn try_from(tool: ToolFunction) -> Result<Self> {
        let input_schema = match tool.parameters {
            Some(v) => {
                let claude_schema: ClaudeToolSchema = v.try_into()?;
                Some(claude_schema)
            }
            None => None,
        };

        let claude_tool = ClaudeTool {
            name: tool.name,
            description: tool.description,
            input_schema,
        };

        Ok(claude_tool)
    }
}
