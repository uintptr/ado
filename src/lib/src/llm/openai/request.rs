use std::vec;

use serde::Serialize;

use crate::{
    error::Result,
    functions::config::{ConfigFunction, ConfigFunctions},
};

#[derive(Debug, Serialize)]
pub struct OpenAIContent {
    pub content: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct OpenAIFunctionInput {
    #[serde(rename = "type")]
    pub t: String,
    pub call_id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize)]
pub struct OpenAIFunctionOutput {
    #[serde(rename = "type")]
    pub t: String,
    pub call_id: String,
    pub output: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum OpenAIInput {
    Content(OpenAIContent),
    FunctionInput(OpenAIFunctionInput),
    FunctionOutput(OpenAIFunctionOutput),
}

#[derive(Debug, Serialize)]
pub struct OpenAIFunctionRequest<'a> {
    model: &'a str,
    input: Vec<OpenAIInput>,
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
        let content = OpenAIContent {
            content: content.to_string(),
            role: role.to_string(),
        };

        self.input.push(OpenAIInput::Content(content))
    }

    pub fn reset_input(&mut self) {
        self.input = vec![];
    }

    pub fn with_inputs(&mut self, inputs: Vec<OpenAIInput>) {
        self.input.extend(inputs)
    }

    pub fn to_json(&self) -> Result<String> {
        let json_str = serde_json::to_string_pretty(self)?;
        Ok(json_str)
    }
}
