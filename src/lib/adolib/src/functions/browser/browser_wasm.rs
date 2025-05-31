use crate::{
    error::{Error, Result},
    functions::function_args::FunctionArgs,
};

pub struct FunctionsBrowser {}

impl FunctionsBrowser {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn browse(&self, _args: &FunctionArgs) -> Result<String> {
        Err(Error::NotImplemented)
    }
}
