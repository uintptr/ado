use async_trait::async_trait;
use omcp::types::{BakedMcpToolTrait, McpParams};

use crate::error::{Error, Result};

pub struct ToolGetIpAddress;
pub struct ToolWhoisQuery;

impl ToolGetIpAddress {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolGetIpAddress {
    type Error = Error;

    async fn call(&mut self, _params: &McpParams) -> Result<String> {
        Err(Error::NotImplemented)
    }
}

impl ToolWhoisQuery {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolWhoisQuery {
    type Error = Error;

    async fn call(&mut self, _params: &McpParams) -> Result<String> {
        Err(Error::NotImplemented)
    }
}
