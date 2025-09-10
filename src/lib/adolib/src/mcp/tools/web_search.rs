use async_trait::async_trait;
use log::info;
use omcp::{client::types::BakedMcpToolTrait, types::McpParams};

use crate::error::{Error, Result};

pub struct ToolWebSearch {}

impl ToolWebSearch {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolWebSearch {
    type Error = Error;

    async fn call(&mut self, params: &McpParams) -> Result<String> {
        info!("Hello from {}", params.tool_name);
        Ok("".into())
    }
}
