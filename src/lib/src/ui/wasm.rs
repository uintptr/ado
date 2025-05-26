use crate::error::{Error, Result};

use super::UiTrait;

#[derive(Default)]
pub struct WasmUI {}

impl WasmUI {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl UiTrait for WasmUI {
    fn display(&self, _text: &str) -> Result<()> {
        Err(Error::NotImplemented)
    }

    fn readline(&mut self) -> Result<String> {
        Err(Error::NotImplemented)
    }
}
