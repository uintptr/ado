use async_trait::async_trait;
use omcp::types::{BakedMcpToolTrait, McpParams};

use crate::error::{Error, Result};

pub struct ToolBrowse;

impl ToolBrowse {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolBrowse {
    type Error = Error;

    async fn call(&mut self, _params: &McpParams) -> Result<String> {
        Err(Error::NotImplemented)
    }
}
