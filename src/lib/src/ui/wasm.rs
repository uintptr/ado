use crate::error::Result;

use super::UiTrait;

#[derive(Default)]
pub struct WasmUI {}

impl WasmUI {
    pub fn new() -> Self {
        Self {}
    }
}

impl UiTrait for WasmUI {
    fn display_text(&self, _text: &str) -> Result<()> {
        Ok(())
    }
}
