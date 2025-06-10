#[cfg(target_arch = "wasm32")]
mod js;

#[cfg(not(target_arch = "wasm32"))]
mod console;

pub mod logger {
    #[cfg(target_arch = "wasm32")]
    pub use super::js::setup_wasm_logger as setup_logger;

    #[cfg(not(target_arch = "wasm32"))]
    pub use super::console::setup_console_logger as setup_logger;
}
