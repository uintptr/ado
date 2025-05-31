use crate::{
    error::{Error, Result},
    functions::function_args::FunctionArgs,
};

pub struct FunctionsWhois {}

impl FunctionsWhois {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn query(&self, _args: &FunctionArgs) -> Result<String> {
        Err(Error::NotImplemented)
    }
}
