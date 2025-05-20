use std::collections::HashMap;

use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    functions::function_llm::FunctionLlmGenerate,
};

use super::{function_files::FunctionWriteFile, function_whois::FunctionWhoisExists};

pub trait FunctionTrait {
    fn exec(&self) -> Result<String>;
}

enum FunctionExecutor {
    WhoisExists(FunctionWhoisExists),
    LlmGenerate(FunctionLlmGenerate),
    WriteFile(FunctionWriteFile),
}

impl FunctionExecutor {
    pub fn parse(call: &FunctionCall) -> Result<Self> {
        info!("user calling function={}", call.name);

        match call.name.as_str() {
            "whois_exists" => {
                let f = FunctionWhoisExists::from_args(&call.args)?;
                Ok(FunctionExecutor::WhoisExists(f))
            }
            "generate_text" => {
                let f = FunctionLlmGenerate::from_args(&call.args)?;
                Ok(FunctionExecutor::LlmGenerate(f))
            }
            "write_file" => {
                let f = FunctionWriteFile::from_args(&call.args)?;
                Ok(FunctionExecutor::WriteFile(f))
            }
            f => {
                return Err(Error::UnknownFunction {
                    name: f.to_string(),
                });
            }
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub args: HashMap<String, String>,
}

impl FunctionCall {
    pub fn execute(&self) -> Result<String> {
        let exe = FunctionExecutor::parse(self)?;

        match exe {
            FunctionExecutor::WhoisExists(f) => f.exec(),
            FunctionExecutor::LlmGenerate(f) => f.exec(),
            FunctionExecutor::WriteFile(f) => f.exec(),
        }
    }
}
