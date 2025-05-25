mod assets;
pub mod config;
mod function_args;
pub mod function_handler;
#[cfg(not(target_arch = "wasm32"))]
mod functions_browser;
#[cfg(target_arch = "wasm32")]
mod functions_browser_wasm;
mod functions_files;
mod functions_http;
mod functions_search;
mod functions_shell;
#[cfg(not(target_arch = "wasm32"))]
mod functions_whois;
#[cfg(target_arch = "wasm32")]
mod functions_whois_wasm;