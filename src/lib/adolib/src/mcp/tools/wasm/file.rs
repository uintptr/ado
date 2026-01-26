use async_trait::async_trait;
use omcp::types::{BakedMcpToolTrait, McpParams};

use crate::error::{Error, Result};

pub struct ToolFileRead;
pub struct ToolFileWrite;
pub struct ToolFileFind;
pub struct ToolFileList;

impl ToolFileRead {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolFileRead {
    type Error = Error;

    async fn call(&mut self, _params: &McpParams) -> Result<String> {
        Err(Error::NotImplemented)
    }
}

impl ToolFileWrite {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolFileWrite {
    type Error = Error;

    async fn call(&mut self, _params: &McpParams) -> Result<String> {
        Err(Error::NotImplemented)
    }
}

impl ToolFileFind {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolFileFind {
    type Error = Error;

    async fn call(&mut self, _params: &McpParams) -> Result<String> {
        Err(Error::NotImplemented)
    }
}

impl ToolFileList {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolFileList {
    type Error = Error;

    async fn call(&mut self, _params: &McpParams) -> Result<String> {
        Err(Error::NotImplemented)
    }
}
