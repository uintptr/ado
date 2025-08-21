use crate::{
    data::types::AdoData,
    error::{Error, Result},
    tools::function_args::ToolArgs,
};

pub struct FunctionsWhois {}

impl FunctionsWhois {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn query(&self, _args: &ToolArgs) -> Result<AdoData> {
        Err(Error::NotImplemented)
    }
}
