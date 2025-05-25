use serde::{Deserialize, Deserializer};
use serde_json::Value;

use crate::{
    console::ConsoleUI,
    error::{Error, Result},
    functions::function_handler::FunctionHandler,
};

use super::request::{OpenAIFunctionInput, OpenAIFunctionOutput, OpenAIInput};
use log::error;

#[derive(Debug, Deserialize)]
pub struct OpenAiOutputMessageContent {
    #[serde(rename = "type")]
    pub t: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIOutputMessage {
    pub id: String,
    pub status: String,
    pub content: Vec<OpenAiOutputMessageContent>,
    pub role: String,
}

impl OpenAIOutputMessage {
    pub fn process(&self, console: &ConsoleUI) {
        for c in self.content.iter() {
            if let Err(e) = console.display_text(&c.text) {
                error!("{e}");
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OpenAIOutputFunctionCall {
    pub id: String,
    pub status: String,
    pub arguments: String,
    pub call_id: String,
    pub name: String,
}

impl OpenAIOutputFunctionCall {
    pub fn process(&self, handler: &FunctionHandler) -> Result<String> {
        handler.call(&self.name, &self.arguments)
    }
}

#[derive(Debug, Deserialize)]
pub enum OpenAIOutput {
    Message(OpenAIOutputMessage),
    FunctionCall(OpenAIOutputFunctionCall),
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
    #[serde(deserialize_with = "deserialized_openai_output")]
    pub output: Vec<OpenAIOutput>,
    pub service_tier: String,
    pub store: bool,
    pub temperature: f64,
}

fn deserialized_openai_output<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<OpenAIOutput>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut outputs = Vec::new();

    let values: Vec<Value> = Deserialize::deserialize(deserializer)?;

    for v in values {
        let t = match v.get("type") {
            Some(v) => v,
            None => {
                return Err(serde::de::Error::custom(Error::TypeMissing {
                    t: "type missing".to_string(),
                }));
            }
        };

        let type_str = match t.as_str() {
            Some(v) => v,
            None => {
                return Err(serde::de::Error::custom(Error::TypeMissing {
                    t: "not a string".to_string(),
                }));
            }
        };

        let output = match type_str {
            "function_call" => {
                let func: OpenAIOutputFunctionCall = match serde_json::from_value(v) {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(serde::de::Error::custom(e));
                    }
                };
                OpenAIOutput::FunctionCall(func)
            }
            "message" => {
                let msg: OpenAIOutputMessage = match serde_json::from_value(v) {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(serde::de::Error::custom(e));
                    }
                };
                OpenAIOutput::Message(msg)
            }
            _ => {
                return Err(serde::de::Error::custom(Error::TypeMissing {
                    t: "not implemented".to_string(),
                }));
            }
        };

        outputs.push(output);
    }

    Ok(outputs)
}

impl OpenAIFunctionResponse {
    pub fn from_string(input: &str) -> Result<OpenAIFunctionResponse> {
        let res = serde_json::from_str(input)?;
        Ok(res)
    }

    pub fn process_output(
        &self,
        console: &ConsoleUI,
        func_handler: &FunctionHandler,
    ) -> Result<Vec<OpenAIInput>> {
        let mut inputs = Vec::new();

        for output in self.output.iter() {
            match output {
                OpenAIOutput::Message(m) => m.process(console),
                OpenAIOutput::FunctionCall(f) => {
                    let output = match f.process(func_handler) {
                        Ok(v) => v,
                        Err(e) => format!("error: {e}"),
                    };

                    let out_func = OpenAIFunctionOutput {
                        t: "function_call_output".to_string(),
                        call_id: f.call_id.to_string(),
                        output,
                    };

                    let in_func = OpenAIFunctionInput {
                        t: "function_call".to_string(),
                        call_id: f.call_id.to_string(),
                        name: f.name.to_string(),
                        arguments: f.arguments.to_string(),
                    };

                    inputs.push(OpenAIInput::FunctionOutput(out_func));
                    inputs.push(OpenAIInput::FunctionInput(in_func))
                }
            }
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
        let console = ConsoleUI::new();

        let inputs = res.process_output(&console, &handler).unwrap();

        let res_json = serde_json::to_string_pretty(&inputs).unwrap();

        println!("{res_json}");
    }

    #[test]
    fn test_resp_final() {
        let rel_test = Path::new("test").join("openai_response_final.json");

        let test_file = find_file(rel_test).unwrap();

        let resp_json = fs::read_to_string(test_file).unwrap();

        let res = OpenAIFunctionResponse::from_string(&resp_json).unwrap();
        dbg!(res);
    }
}
