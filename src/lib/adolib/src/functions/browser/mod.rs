#[cfg(target_arch = "wasm32")]
mod browser_wasm;

#[cfg(not(target_arch = "wasm32"))]
mod browser_metal;

pub mod functions {
    #[cfg(not(target_arch = "wasm32"))]
    pub use super::browser_metal::FunctionsBrowser;
    #[cfg(target_arch = "wasm32")]
    pub use super::browser_wasm::FunctionsBrowser;
}
