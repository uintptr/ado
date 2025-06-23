use crate::{
    data::AdoData,
    error::{Error, Result},
    tools::function_args::FunctionArgs,
};

pub struct FunctionsBrowser {}

impl FunctionsBrowser {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn browse(&self, _args: &FunctionArgs) -> Result<AdoData> {
        Err(Error::NotImplemented)
    }
}
