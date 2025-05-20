use std::vec;

use serde::{Deserialize, Serialize};

use crate::{
    error::Result,
    functions::{
        config::{ConfigFunction, ConfigFunctions},
        function_handler::FunctionCall,
    },
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FunctionResponse {
    result: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FunctionResponsePart {
    name: String,
    response: FunctionResponse,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "functionResponse", skip_serializing_if = "Option::is_none")]
    pub response: Option<FunctionResponsePart>,
    #[serde(rename = "functionCall", skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
pub struct Tool<'a> {
    #[serde(rename = "functionDeclarations")]
    functions: &'a [ConfigFunction],
}

#[derive(Debug, Serialize)]
pub struct Contents<'a> {
    contents: Vec<&'a Content>,
    tools: Vec<Tool<'a>>,
}

pub struct ContentBuilder(Content);

impl<'c> ContentBuilder {
    pub fn new<S>(role: S) -> ContentBuilder
    where
        S: AsRef<str>,
    {
        let c = Content {
            role: role.as_ref().to_string(),
            ..Default::default()
        };
        ContentBuilder(c)
    }

    pub fn with_text<S>(mut self, text: S) -> Self
    where
        S: AsRef<str>,
    {
        let p = Part {
            text: Some(text.as_ref().to_string()),
            response: None,
            function_call: None,
        };

        self.0.parts.push(p);

        self
    }

    pub fn with_response<S>(mut self, command: S, result: S) -> Self
    where
        S: AsRef<str>,
    {
        let response = FunctionResponse {
            result: result.as_ref().to_string(),
        };

        let response_part = FunctionResponsePart {
            name: command.as_ref().to_string(),
            response,
        };

        let p = Part {
            text: None,
            response: Some(response_part),
            function_call: None,
        };

        self.0.parts.push(p);

        self
    }

    pub fn build(self) -> Content {
        self.0
    }
}

impl<'a, 'b> Contents<'a> {
    pub fn new() -> Self {
        Contents {
            contents: Vec::new(),
            tools: vec![],
        }
    }

    pub fn with_functions(&mut self, functions: &'a ConfigFunctions) {
        let tool = Tool {
            functions: &functions.list,
        };

        self.tools = vec![tool];
    }

    pub fn with_content(&mut self, content: &'a Content) {
        self.contents.push(content)
    }

    pub fn to_json(&self) -> Result<String> {
        let json_string = serde_json::to_string_pretty(self)?;
        Ok(json_string)
    }

    pub fn parse_call(&self, _call: &FunctionCall) -> Result<String> {
        todo!()
    }
}
