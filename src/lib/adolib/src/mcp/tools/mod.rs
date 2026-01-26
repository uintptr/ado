pub mod http;
pub mod web_search;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
pub use wasm::{
    browse::ToolBrowse,
    file::{ToolFileFind, ToolFileList, ToolFileRead, ToolFileWrite},
    network::{ToolGetIpAddress, ToolWhoisQuery},
    shell::ToolShellExec,
};

#[cfg(not(target_arch = "wasm32"))]
pub use native::{
    browse::ToolBrowse,
    file::{ToolFileFind, ToolFileList, ToolFileRead, ToolFileWrite},
    network::{ToolGetIpAddress, ToolWhoisQuery},
    shell::ToolShellExec,
};
