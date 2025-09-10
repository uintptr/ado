use async_trait::async_trait;
use log::info;
use omcp::{client::types::BakedMcpToolTrait, types::McpParams};

use crate::error::{Error, Result};

pub struct ToolShellExec {}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////
// GET IP
///////////////////////////////////////

impl ToolShellExec {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolShellExec {
    type Error = Error;

    async fn call(&mut self, params: &McpParams) -> Result<String> {
        info!("Hello from {}", params.tool_name);
        Ok("".into())
    }
}
