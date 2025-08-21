use crate::{
    data::types::AdoData,
    error::{Error, Result},
    tools::function_args::ToolArgs,
};

pub struct FunctionsBrowser {}

impl FunctionsBrowser {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn browse(&self, _args: &ToolArgs) -> Result<AdoData> {
        Err(Error::NotImplemented)
    }
}
