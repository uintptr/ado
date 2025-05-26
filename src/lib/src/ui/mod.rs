use crate::error::Result;

pub trait UiTrait {
    fn display(&self, _text: &str) -> Result<()>;
    fn readline(&mut self) -> Result<String>;
}

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub mod ui {
    pub use super::wasm::WasmUI as Console;
}

#[cfg(not(target_arch = "wasm32"))]
mod console;

#[cfg(not(target_arch = "wasm32"))]
pub mod ui {
    pub use super::console::ConsoleUI as Console;
}
