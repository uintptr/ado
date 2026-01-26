use async_trait::async_trait;
use omcp::types::{BakedMcpToolTrait, McpParams};

use crate::error::{Error, Result};

pub struct ToolShellExec;

impl ToolShellExec {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolShellExec {
    type Error = Error;

    async fn call(&mut self, _params: &McpParams) -> Result<String> {
        Err(Error::NotImplemented)
    }
}
