use crate::error::Result;

pub trait UiTrait {
    fn display(&self, _text: &str) -> Result<()>;
    fn read_input(&mut self) -> Result<String>;
}

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
mod console;

pub mod ui {
    #[cfg(not(target_arch = "wasm32"))]
    pub use super::console::ConsoleUI as Console;
    #[cfg(target_arch = "wasm32")]
    pub use super::wasm::WasmUI as Console;
}

mod user_commands;
