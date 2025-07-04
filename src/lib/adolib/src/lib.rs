pub mod config;
pub mod const_vars;
pub mod data;
pub mod error;
pub mod http;
pub mod llm;
pub mod logging;
pub mod search;
pub mod shell;
pub mod staples;
pub mod tools;
pub mod ui;
#[cfg(target_arch = "wasm32")]
pub mod wasm;
