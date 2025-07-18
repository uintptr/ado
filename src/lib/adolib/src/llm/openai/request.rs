use std::vec;

use serde::Serialize;

use crate::{
    error::Result,
    tools::config::{ConfigFunction, ConfigFunctions},
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
pub struct OpenAIRequest {
    model: String,
    input: Vec<OpenAIInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ConfigFunction>>,
}

impl OpenAIRequest {
    pub fn new(model: &str) -> OpenAIRequest {
        OpenAIRequest {
            model: model.to_string(),
            input: vec![],
            tools: None,
        }
    }

    pub fn with_functions(&mut self, functions: ConfigFunctions) {
        let mut funcs = Vec::new();

        for f in &functions.list {
            funcs.push(f.name.clone());
        }

        let prompt = format!(
            "Don't forget that you have access to functions. The functions are {}",
            funcs.join(",")
        );

        self.with_input_role("user", prompt);

        self.tools = Some(functions.list)
    }

    pub fn with_input_role<S1, S2>(&mut self, role: S1, content: S2)
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        let content = OpenAIContent {
            content: content.as_ref().to_string(),
            role: role.as_ref().to_string(),
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
