use serde::Deserialize;
use std::collections::HashMap;

use crate::{
    error::{Error, Result},
    functions::function_handler::FunctionHandler,
};

use log::error;

#[derive(Debug, Deserialize)]
pub struct OpenAiContent {
    #[serde(rename = "type")]
    pub t: String,
    pub text: String,
}

#[derive(Default)]
struct OpenAIFunctionDef {
    pub id: String,
    pub arguments: String,
    pub call_id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIFunctionResponseEntry {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub t: String,
    pub id: String,
    pub status: String,
    pub call_id: Option<String>,
    pub arguments: Option<String>,
    pub role: Option<String>,
    pub content: Option<Vec<OpenAiContent>>,
}

impl TryFrom<&OpenAIFunctionResponseEntry> for OpenAIFunctionDef {
    type Error = Error;

    fn try_from(value: &OpenAIFunctionResponseEntry) -> Result<Self> {
        let mut def = OpenAIFunctionDef::default();

        if value.t != "function_call" {
            return Err(Error::TypeError {
                error: "Not a function_call".into(),
            });
        }

        def.id = value.id.to_string();

        def.arguments = match &value.arguments {
            Some(v) => v.to_string(),
            None => {
                return Err(Error::TypeMissing {
                    t: "arguments".to_string(),
                });
            }
        };

        def.call_id = match &value.call_id {
            Some(v) => v.to_string(),
            None => {
                return Err(Error::TypeMissing {
                    t: "call_id".to_string(),
                });
            }
        };

        def.name = match &value.name {
            Some(v) => v.to_string(),
            None => {
                return Err(Error::TypeMissing {
                    t: "name".to_string(),
                });
            }
        };

        Ok(def)
    }
}

#[derive(Debug, Deserialize)]
pub struct OpenAIFunctionResponse {
    pub id: String,
    pub created_at: u64,
    pub status: String,
    pub error: Option<String>,
    pub incomplete_details: Option<String>,
    pub instructions: Option<String>,
    pub model: String,
    pub parallel_tool_calls: bool,
    pub previous_response_id: Option<String>,
    pub output: Vec<OpenAIFunctionResponseEntry>,
    pub service_tier: String,
    pub store: bool,
    pub temperature: f64,
}

impl OpenAIFunctionResponse {
    pub fn from_string(input: &str) -> Result<OpenAIFunctionResponse> {
        let res = serde_json::from_str(input)?;
        Ok(res)
    }

    pub fn content_text(&self) -> Result<String> {
        for o in &self.output {
            if let Some(content_list) = &o.content {
                for content in content_list {
                    if content.t == "output_text" {
                        return Ok(content.text.to_string());
                    }
                }
            }
        }

        Err(Error::ContentTextNotFound)
    }

    pub fn is_function_call(&self) -> bool {
        let output = match self.output.first() {
            Some(v) => v,
            None => {
                return false;
            }
        };

        output.t == "function_call"
    }

    pub fn call_functions(
        &self,
        handler: &FunctionHandler,
    ) -> Result<Vec<HashMap<String, String>>> {
        let mut inputs = Vec::new();

        for f in self.output.iter() {
            let f: OpenAIFunctionDef = match f.try_into() {
                Ok(v) => v,
                Err(e) => {
                    error!("{e}");
                    continue;
                }
            };

            let args_map: HashMap<String, String> = serde_json::from_str(&f.arguments)?;

            let output = match handler.call(&f.name, &args_map) {
                Ok(v) => v,
                Err(e) => format!("error: {e}"),
            };

            let mut function_output: HashMap<String, String> = HashMap::new();

            function_output.insert("output".into(), output);
            function_output.insert("call_id".into(), f.call_id.to_string());
            function_output.insert("type".into(), "function_call_output".into());

            let mut function_call: HashMap<String, String> = HashMap::new();

            function_call.insert("type".into(), "function_call".into());
            function_call.insert("call_id".into(), f.call_id.to_string());
            function_call.insert("name".into(), f.name.to_string());
            function_call.insert("arguments".into(), f.arguments.to_string());

            inputs.push(function_output);
            inputs.push(function_call);
        }

        Ok(inputs)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use crate::staples::find_file;

    use super::*;

    #[test]
    fn test_resp_1() {
        let rel_test = Path::new("test").join("openai_response.json");

        let test_file = find_file(rel_test).unwrap();

        let resp_json = fs::read_to_string(test_file).unwrap();

        let res = OpenAIFunctionResponse::from_string(&resp_json).unwrap();

        let handler = FunctionHandler::new().unwrap();

        let inputs = res.call_functions(&handler).unwrap();

        let res_json = serde_json::to_string_pretty(&inputs).unwrap();

        println!("{res_json}");
    }

    #[test]
    fn test_resp_final() {
        let rel_test = Path::new("test").join("openai_response_final.json");

        let test_file = find_file(rel_test).unwrap();

        let resp_json = fs::read_to_string(test_file).unwrap();

        let res = OpenAIFunctionResponse::from_string(&resp_json).unwrap();

        let text = res.content_text().unwrap();

        println!("{text}");

        dbg!(res);
    }
}
