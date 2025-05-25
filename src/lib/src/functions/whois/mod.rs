#[cfg(target_arch = "wasm32")]
mod whois_wasm;

#[cfg(not(target_arch = "wasm32"))]
mod whois_metal;

pub mod whois {
    #[cfg(not(target_arch = "wasm32"))]
    pub use super::whois_metal::FunctionsWhois;
    #[cfg(target_arch = "wasm32")]
    pub use super::whois_wasm::FunctionsWhois;
}
