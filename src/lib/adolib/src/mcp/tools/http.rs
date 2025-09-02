use log::info;
use omcp::{client::types::BakedMcpToolTrait, types::McpParams};

use crate::error::{Error, Result};

pub struct ToolHttpGet {}
pub struct ToolHttpPost {}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////
// HTTP GET
///////////////////////////////////////

impl ToolHttpGet {
    pub fn new() -> Self {
        Self {}
    }
}

impl BakedMcpToolTrait for ToolHttpGet {
    type Error = Error;

    fn call(&mut self, params: &McpParams) -> Result<String> {
        info!("Hello from {}", params.tool_name);
        Ok("".into())
    }
}

///////////////////////////////////////
// HTTP GET
///////////////////////////////////////

impl ToolHttpPost {
    pub fn new() -> Self {
        Self {}
    }
}

impl BakedMcpToolTrait for ToolHttpPost {
    type Error = Error;

    fn call(&mut self, params: &McpParams) -> Result<String> {
        info!("Hello from {}", params.tool_name);
        Ok("".into())
    }
}
