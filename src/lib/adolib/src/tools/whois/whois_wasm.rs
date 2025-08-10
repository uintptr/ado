use crate::{
    data::types::AdoData,
    error::{Error, Result},
    tools::function_args::FunctionArgs,
};

pub struct FunctionsWhois {}

impl FunctionsWhois {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn query(&self, _args: &FunctionArgs) -> Result<AdoData> {
        Err(Error::NotImplemented)
    }
}
