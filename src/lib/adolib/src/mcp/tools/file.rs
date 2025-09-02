use log::info;
use omcp::{client::types::BakedMcpToolTrait, types::McpParams};

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

impl ToolFileRead {
    pub fn new() -> Self {
        Self {}
    }
}

impl BakedMcpToolTrait for ToolFileRead {
    type Error = Error;

    fn call(&mut self, params: &McpParams) -> Result<String> {
        info!("Hello from {}", params.tool_name);
        Ok("".into())
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

impl BakedMcpToolTrait for ToolFileWrite {
    type Error = Error;

    fn call(&mut self, params: &McpParams) -> Result<String> {
        info!("Hello from {}", params.tool_name);
        Ok("".into())
    }
}

///////////////////////////////////////
// FIND
///////////////////////////////////////

impl ToolFileFind {
    pub fn new() -> Self {
        Self {}
    }
}

impl BakedMcpToolTrait for ToolFileFind {
    type Error = Error;

    fn call(&mut self, params: &McpParams) -> Result<String> {
        info!("Hello from {}", params.tool_name);
        Ok("".into())
    }
}

///////////////////////////////////////
// LIST
///////////////////////////////////////
impl ToolFileList {
    pub fn new() -> Self {
        Self {}
    }
}

impl BakedMcpToolTrait for ToolFileList {
    type Error = Error;

    fn call(&mut self, params: &McpParams) -> Result<String> {
        info!("Hello from {}", params.tool_name);
        Ok("".into())
    }
}
