use log::info;
use omcp::{client::types::BakedMcpToolTrait, types::McpParams};

use crate::error::{Error, Result};

pub struct ToolBrowse {}

impl ToolBrowse {
    pub fn new() -> Self {
        Self {}
    }
}

impl BakedMcpToolTrait for ToolBrowse {
    type Error = Error;

    fn call(&mut self, params: &McpParams) -> Result<String> {
        info!("Hello from {}", params.tool_name);
        Ok("".into())
    }
}
