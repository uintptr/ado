pub mod file;

#[cfg(not(target_arch = "wasm32"))]
mod disk;
#[cfg(target_arch = "wasm32")]
mod wasm;
