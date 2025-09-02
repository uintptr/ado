use log::info;
use omcp::{client::types::BakedMcpToolTrait, types::McpParams};

use crate::error::{Error, Result};

pub struct ToolGetIpAddress {}
pub struct ToolWhoisQuery {}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////
// GET IP
///////////////////////////////////////

impl ToolGetIpAddress {
    pub fn new() -> Self {
        Self {}
    }
}

impl BakedMcpToolTrait for ToolGetIpAddress {
    type Error = Error;

    fn call(&mut self, params: &McpParams) -> Result<String> {
        info!("Hello from {}", params.tool_name);
        Ok("".into())
    }
}

///////////////////////////////////////
// WHOIS
///////////////////////////////////////

impl ToolWhoisQuery {
    pub fn new() -> Self {
        Self {}
    }
}

impl BakedMcpToolTrait for ToolWhoisQuery {
    type Error = Error;

    fn call(&mut self, params: &McpParams) -> Result<String> {
        info!("Hello from {}", params.tool_name);
        Ok("".into())
    }
}
