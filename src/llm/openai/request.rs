use std::{collections::HashMap, vec};

use serde::Serialize;

use crate::{
    error::Result,
    functions::config::{ConfigFunction, ConfigFunctions},
};

/*
#[derive(Debug, Default, Serialize)]
pub struct OpenAIInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    t: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<String>,
}

impl OpenAIInput {
    pub fn new() -> OpenAIInput {
        OpenAIInput::default()
    }

    pub fn with_role(mut self, role: &str) -> Self {
        self.role = Some(role.to_string());
        self
    }

    pub fn with_content(mut self, content: &str) -> Self {
        self.content = Some(content.to_string());
        self
    }

    pub fn with_type(mut self, t: &str) -> Self {
        self.t = Some(t.to_string());
        self
    }

    pub fn with_call_id(mut self, call_id: &str) -> Self {
        self.call_id = Some(call_id.to_string());
        self
    }

    pub fn with_output(mut self, output: &str) -> Self {
        self.output = Some(output.to_string());
        self
    }
}
    */

#[derive(Debug, Serialize)]
pub struct OpenAIFunctionRequest<'a> {
    model: &'a str,
    input: Vec<HashMap<String, String>>,
    tools: &'a Vec<ConfigFunction>,
}

impl<'a> OpenAIFunctionRequest<'a> {
    pub fn new(model: &'a str, functions: &'a ConfigFunctions) -> Self {
        Self {
            model,
            input: vec![],
            tools: &functions.list,
        }
    }

    pub fn with_input_role(&mut self, role: &str, content: &str) {
        let mut map: HashMap<String, String> = HashMap::new();

        map.insert("role".to_string(), role.to_string());
        map.insert("content".to_string(), content.to_string());
        self.input.push(map)
    }

    pub fn with_inputs(&mut self, inputs: Vec<HashMap<String, String>>) {
        for input in inputs {
            self.input.push(input)
        }
    }

    pub fn to_json(&self) -> Result<String> {
        let json_str = serde_json::to_string_pretty(self)?;
        Ok(json_str)
    }
}
