use serde::{Deserialize, Deserializer};
use serde_json::Value;

use crate::{
    data::types::AdoData,
    error::{Error, Result},
    tools::handler::FunctionHandler,
};

use super::request::{OpenAIFunctionInput, OpenAIFunctionOutput, OpenAIInput};

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

#[derive(Debug, Deserialize)]
pub struct OpenAIOutputFunctionCall {
    pub id: String,
    pub status: String,
    pub arguments: String,
    pub call_id: String,
    pub name: String,
}

impl OpenAIOutputFunctionCall {
    pub async fn process(&self, handler: &FunctionHandler) -> Result<AdoData> {
        handler.call(&self.name, &self.arguments).await
    }
}

#[derive(Debug, Deserialize)]
pub enum OpenAIResponseOutput {
    Message(OpenAIOutputMessage),
    FunctionCall(OpenAIOutputFunctionCall),
}

#[derive(Debug, Deserialize)]
pub struct OpenAIResponse {
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
    pub output: Vec<OpenAIResponseOutput>,
    pub service_tier: String,
    pub store: bool,
    pub temperature: f64,
}

fn deserialized_openai_output<'de, D>(deserializer: D) -> std::result::Result<Vec<OpenAIResponseOutput>, D::Error>
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
                OpenAIResponseOutput::FunctionCall(func)
            }

            "message" => {
                let msg: OpenAIOutputMessage = match serde_json::from_value(v) {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(serde::de::Error::custom(e));
                    }
                };
                OpenAIResponseOutput::Message(msg)
            }
            "reasoning" => {
                continue;
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

pub struct OpenAIOutput {
    pub message: AdoData,
    pub inputs: Vec<OpenAIInput>,
}

impl OpenAIResponse {
    pub fn from_string(input: &str) -> Result<OpenAIResponse> {
        let res = serde_json::from_str(input)?;
        Ok(res)
    }

    pub async fn process_output(&self, func_handler: &FunctionHandler) -> Result<OpenAIOutput> {
        let mut message = AdoData::String("".to_string());
        let mut inputs = Vec::new();

        for output in self.output.iter() {
            match output {
                OpenAIResponseOutput::Message(m) => {
                    assert!(1 == m.content.len());

                    if let Some(content) = m.content.first() {
                        message = AdoData::String(content.text.to_string());
                        break;
                    }
                }
                OpenAIResponseOutput::FunctionCall(f) => {
                    let output = match f.process(func_handler).await {
                        Ok(v) => v,
                        Err(e) => AdoData::String(format!("error: {e}")),
                    };

                    let output: String = output.try_into()?;

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

        Ok(OpenAIOutput { message, inputs })
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use rstaples::staples::find_file;

    use crate::config::loader::ConfigFile;

    use super::*;

    #[tokio::test]
    async fn test_resp_1() {
        let rel_test = Path::new("test").join("openai_response.json");

        let test_file = find_file(rel_test).unwrap();

        let resp_json = fs::read_to_string(test_file).unwrap();

        let res = OpenAIResponse::from_string(&resp_json).unwrap();

        let config = ConfigFile::from_disk().unwrap();

        let handler = FunctionHandler::new(&config).unwrap();

        res.process_output(&handler).await.unwrap();
    }

    #[test]
    fn test_resp_final() {
        let rel_test = Path::new("test").join("openai_response_final.json");

        let test_file = find_file(rel_test).unwrap();

        let resp_json = fs::read_to_string(test_file).unwrap();

        let res = OpenAIResponse::from_string(&resp_json).unwrap();
        dbg!(res);
    }
}
