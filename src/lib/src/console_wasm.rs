use crate::error::Result;

#[derive(Default)]
pub struct ConsoleUI {}

impl ConsoleUI {
    pub fn new() -> Self {
        Self {}
    }

    pub fn display_text(&self, _text: &str) -> Result<()> {
        Ok(())
    }
}
