use std::{env, os::unix::fs::FileTypeExt, path::PathBuf, time::UNIX_EPOCH};

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose};
use log::{error, info};
use omcp::{client::types::BakedMcpToolTrait, types::McpParams};
use serde::Serialize;
use tokio::fs;
use walkdir::{DirEntry, WalkDir};

use crate::error::{Error, Result};

pub struct ToolFileRead {}
pub struct ToolFileWrite {}
pub struct ToolFileFind {}
pub struct ToolFileList {}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////
// READ
///////////////////////////////////////

const FILE_READ_MAX_SIZE: u64 = 500 * 1024;

#[derive(Serialize)]
struct ToolFileReadResult {
    file_path: PathBuf,
    file_size: u64,
    base64_data: String,
}

impl ToolFileRead {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolFileRead {
    type Error = Error;

    async fn call(&mut self, params: &McpParams) -> Result<String> {
        let file_path = PathBuf::from(params.get_string("file_path")?);

        info!("reading {}", file_path.display());

        if !file_path.exists() {
            return Err(Error::FileNotFoundError { file_path });
        }

        let meta = fs::metadata(&file_path).await?;

        let file_size = meta.len();

        if file_size > FILE_READ_MAX_SIZE {
            return Err(Error::FileTooLarge {
                size: file_size,
                limit: FILE_READ_MAX_SIZE,
            });
        }

        let file_data = fs::read(&file_path).await?;

        let base64_data = general_purpose::STANDARD.encode(file_data);

        let result = ToolFileReadResult {
            file_path,
            file_size,
            base64_data,
        };

        Ok(serde_json::to_string(&result)?)
    }
}

///////////////////////////////////////
// WRITE
///////////////////////////////////////

impl ToolFileWrite {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolFileWrite {
    type Error = Error;

    async fn call(&mut self, params: &McpParams) -> Result<String> {
        let file_path = PathBuf::from(params.get_string("file_path")?);
        let file_data = params.get_string("file_path")?;
        let file_data = general_purpose::STANDARD.decode(file_data)?;

        info!("writing {} bytes to {}", file_data.len(), file_path.display());

        fs::write(&file_path, &file_data).await?;

        Ok(format!(
            "Successfully written {} bytes to {}",
            file_data.len(),
            file_path.display()
        ))
    }
}

///////////////////////////////////////
// FIND
///////////////////////////////////////

#[derive(Serialize)]
struct ToolFileFindResult {
    files: Vec<PathBuf>,
}

impl ToolFileFind {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolFileFind {
    type Error = Error;

    async fn call(&mut self, params: &McpParams) -> Result<String> {
        let file_name = params.get_string("file_name")?;

        let search_path = match params.get_string("search_path") {
            Ok(v) => PathBuf::from(v),
            Err(_) => env::current_dir()?,
        };

        info!("searching for {file_name} under {}", search_path.display());

        let pattern = format!("{}/**/{file_name}", search_path.display());

        let mut files = Vec::new();

        for f in glob::glob(&pattern)? {
            if let Ok(file) = f {
                files.push(file)
            }
        }

        let result = ToolFileFindResult { files };
        Ok(serde_json::to_string(&result)?)
    }
}

///////////////////////////////////////
// LIST
///////////////////////////////////////

const FILE_FIND_MAX_DEPTH: usize = 10;

#[derive(Default, Serialize)]
struct ToolFileListEntry {
    file_path: PathBuf,
    file_type: &'static str,
    file_size: u64,
    file_permissions: String,
    file_modification_ts: u64,
    file_creation_ts: u64,
}

impl TryFrom<DirEntry> for ToolFileListEntry {
    type Error = Error;

    fn try_from(entry: DirEntry) -> Result<Self> {
        let meta = entry.metadata()?;

        let file_path = PathBuf::from(entry.path());

        let file_type = if meta.file_type().is_dir() {
            "directory"
        } else if meta.file_type().is_file() {
            "file"
        } else if meta.file_type().is_symlink() {
            "symlink"
        } else if meta.file_type().is_block_device() {
            "block device"
        } else if meta.file_type().is_socket() {
            "socket"
        } else {
            "unknown"
        };

        let file_size = meta.len();
        let file_permissions = format!("{:?}", meta.permissions());
        let file_modification_ts = match meta.modified() {
            Ok(v) => {
                let duration = v.duration_since(UNIX_EPOCH).unwrap_or_default();
                duration.as_secs()
            }
            Err(_) => 0,
        };
        let file_creation_ts = match meta.created() {
            Ok(v) => {
                let duration = v.duration_since(UNIX_EPOCH).unwrap_or_default();
                duration.as_secs()
            }
            Err(_) => 0,
        };

        Ok(ToolFileListEntry {
            file_path,
            file_type,
            file_size,
            file_permissions,
            file_modification_ts,
            file_creation_ts,
        })
    }
}

#[derive(Serialize)]
struct ToolFileListResult {
    files: Vec<ToolFileListEntry>,
}

impl ToolFileList {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolFileList {
    type Error = Error;

    async fn call(&mut self, params: &McpParams) -> Result<String> {
        let directory = PathBuf::from(params.get_string("directory")?);
        let show_hidden = match params.get_bool("show_hidden") {
            Ok(v) => v,
            Err(_) => false,
        };
        let recursive = match params.get_bool("recursive") {
            Ok(v) => v,
            Err(_) => false,
        };

        info!(
            "listing directory={} show_hidden={show_hidden} recursive={recursive}",
            directory.display()
        );

        let max_depth = match recursive {
            true => FILE_FIND_MAX_DEPTH,
            false => 1,
        };

        let wd = WalkDir::new(directory).max_depth(max_depth);

        let mut files = Vec::new();

        for entry in wd {
            let entry = match entry {
                Ok(v) => v,
                Err(e) => {
                    error!("{e}");
                    continue;
                }
            };

            let file_entry: ToolFileListEntry = match entry.try_into() {
                Ok(v) => v,
                Err(e) => {
                    error!("{e}");
                    continue;
                }
            };
            files.push(file_entry);
        }

        let result = ToolFileListResult { files };
        Ok(serde_json::to_string(&result)?)
    }
}

#[cfg(test)]
mod tests {
    use omcp::{client::types::BakedMcpToolTrait, types::McpParams};
    use serde_json::Value;

    use crate::{logging::logger::setup_logger, mcp::tools::file::ToolFileList};

    #[tokio::test]
    async fn test_file_list() {
        setup_logger(true).unwrap();

        let mut p = McpParams::new("file_list");
        p.add_argument("directory", Value::String("/tmp".to_string()));

        let mut fl = ToolFileList::new();

        fl.call(&p).await.unwrap();
    }
}
