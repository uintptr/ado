use std::{path::PathBuf, process::Command};

use log::info;
use omcp::{client::types::BakedMcpToolTrait, types::McpParams};
use serde::Serialize;
use which::which;

use crate::error::{Error, Result};

#[derive(Serialize)]
pub struct ToolBrowseResult {
    success: bool,
}

pub struct ToolBrowse {
    xdg_open: PathBuf,
}

impl ToolBrowse {
    pub fn new() -> Result<Self> {
        let xdg_open = which("xdg-open")?;

        Ok(Self { xdg_open })
    }
}

impl BakedMcpToolTrait for ToolBrowse {
    type Error = Error;

    fn call(&mut self, params: &McpParams) -> Result<String> {
        let url = params.get("url")?.as_str().ok_or(Error::InvalidFormat)?;

        info!("browsing to {url}");

        let mut child = Command::new(&self.xdg_open).arg(url).spawn()?;

        let ret = child.wait()?;

        info!("{} returned {:?}", self.xdg_open.display(), ret);

        let result = ToolBrowseResult { success: ret.success() };
        let result = serde_json::to_string(&result)?;

        Ok(result)
    }
}
