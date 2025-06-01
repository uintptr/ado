use crate::error::{Error, Result};

use super::UiTrait;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmUI {}

#[wasm_bindgen]
impl WasmUI {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {}
    }
}

impl UiTrait for WasmUI {
    fn display(&self, _text: &str) -> Result<()> {
        Err(Error::NotImplemented)
    }

    fn display_error(&self, _err: Error) -> Result<()> {
        Err(Error::NotImplemented)
    }

    async fn read_input(&mut self) -> Result<String> {
        Err(Error::NotImplemented)
    }
}
