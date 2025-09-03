use std::{collections::HashMap, sync::Arc};

use log::{error, info};
use omcp::{
    client::{baked::BakedClient, builder::OMcpClientBuilder, io::OMcpClientTrait, types::OMcpServerType},
    types::{McpParams, McpTool, McpTypes},
};
use tokio::sync::Mutex;

use crate::{
    config::loader::AdoConfig,
    data::types::AdoData,
    error::{Error, Result},
    mcp::{
        assets::{McpAssetsAll, McpAssetsPlatform},
        tools::{
            browse::ToolBrowse,
            file::{ToolFileFind, ToolFileList, ToolFileRead, ToolFileWrite},
            http::{ToolHttpGet, ToolHttpPost},
            network::{ToolGetIpAddress, ToolWhoisQuery},
            shell::ToolShellExec,
            web_search::ToolWebSearch,
        },
        types::McpConfig,
    },
};

fn init_sse(name: &str, config: &McpConfig) -> Result<Box<dyn OMcpClientTrait>> {
    let builder = match &config.url {
        Some(v) => OMcpClientBuilder::new(OMcpServerType::Sse).with_sse_url(v),
        None => {
            let err_msg = format!("Missing url in SSE config for {name}");
            return Err(Error::ConfigError { error: err_msg });
        }
    };

    let builder = match &config.authorization_token {
        Some(v) => builder.with_sse_bearer(v)?,
        None => builder,
    };

    let client = builder.build();

    Ok(client)
}

fn load_embedded_platform_tools() -> Vec<McpTool> {
    let mut tools: Vec<McpTool> = Vec::new();

    //
    // Platform specific tools
    //
    for name in McpAssetsPlatform::iter() {
        info!("{name}");

        let e = match McpAssetsPlatform::get(&name) {
            Some(v) => v,
            None => {
                error!("Unable to read {name}");
                continue;
            }
        };

        let json_str = String::from_utf8_lossy(&e.data);

        let embedded_tools: Vec<McpTool> = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(e) => {
                error!("{e}");
                continue;
            }
        };

        tools.extend(embedded_tools);
    }

    tools
}

fn load_embedded_generic_tools() -> Vec<McpTool> {
    let mut tools: Vec<McpTool> = Vec::new();

    //
    // Platform specific tools
    //
    for name in McpAssetsAll::iter() {
        info!("{name}");

        let e = match McpAssetsAll::get(&name) {
            Some(v) => v,
            None => {
                error!("Unable to read {name}");
                continue;
            }
        };

        let json_str = String::from_utf8_lossy(&e.data);

        let embedded_tools: Vec<McpTool> = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(e) => {
                error!("{e}");
                continue;
            }
        };

        tools.extend(embedded_tools);
    }

    tools
}

fn load_embedded_tools() -> Vec<McpTool> {
    let mut tools: Vec<McpTool> = Vec::new();

    tools.extend(load_embedded_platform_tools());
    tools.extend(load_embedded_generic_tools());

    tools
}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////

pub struct McpToolEntry {
    client: Arc<Mutex<Box<dyn OMcpClientTrait>>>,
    tool: McpTool,
}

#[derive(Default)]
pub struct McpMatrix {
    tools: HashMap<String, McpToolEntry>,
}

impl McpToolEntry {
    pub fn new(client: Arc<Mutex<Box<dyn OMcpClientTrait>>>, tool: McpTool) -> Self {
        Self { client, tool }
    }
}

impl McpMatrix {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    fn add_tools(&mut self, client: Box<dyn OMcpClientTrait>, mcp_tools: Vec<McpTool>) {
        let client_arc = Arc::new(Mutex::new(client));

        for tool in mcp_tools {
            let name = tool.name.to_string();

            let entry = McpToolEntry::new(client_arc.clone(), tool);

            self.tools.insert(name, entry);
        }
    }

    async fn load_remote(&mut self, config: &AdoConfig) {
        if let Some(servers) = config.mcp_servers() {
            for (name, config) in servers {
                let mut client = match config.config_type {
                    McpTypes::Sse => match init_sse(name, config) {
                        Ok(v) => v,
                        Err(e) => {
                            error!("Unable to intialize to {name} ({e})");
                            continue;
                        }
                    },
                };

                if let Err(e) = client.connect().await {
                    error!("Unable to connect to {name} ({e})");
                    continue;
                }

                let mcp_tools = match client.list_tools().await {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Unable to list tools for {name} ({e})");

                        if let Err(e) = client.disconnect().await {
                            error!("{e}");
                        }

                        continue;
                    }
                };

                self.add_tools(client, mcp_tools)
            }
        }
    }

    fn load_embedded(&mut self) -> Result<()> {
        info!("Loading Embedded MCP Tools");

        let mcp_tools = load_embedded_tools();

        for t in mcp_tools {
            let name = t.name.clone();

            let client: Box<dyn OMcpClientTrait> = match t.name.as_str() {
                "file_read" => {
                    let tool = ToolFileRead::new();
                    BakedClient::new(tool)
                }
                "file_write" => {
                    let tool = ToolFileWrite::new();
                    BakedClient::new(tool)
                }
                "browse" => {
                    let tool = ToolBrowse::new();
                    BakedClient::new(tool)
                }
                "file_find" => {
                    let tool = ToolFileFind::new();
                    BakedClient::new(tool)
                }
                "file_list" => {
                    let tool = ToolFileList::new();
                    BakedClient::new(tool)
                }
                "get_ip_address" => {
                    let tool = ToolGetIpAddress::new();
                    BakedClient::new(tool)
                }
                "shell_exec" => {
                    let tool = ToolShellExec::new();
                    BakedClient::new(tool)
                }
                "whois_query" => {
                    let tool = ToolWhoisQuery::new();
                    BakedClient::new(tool)
                }
                "http_get" => {
                    let tool = ToolHttpGet::new();
                    BakedClient::new(tool)
                }
                "http_post" => {
                    let tool = ToolHttpPost::new();
                    BakedClient::new(tool)
                }
                "search" => {
                    let tool = ToolWebSearch::new();
                    BakedClient::new(tool)
                }
                unk => {
                    error!("{unk} is not implemented");
                    panic!();
                }
            };

            let client = Arc::new(Mutex::new(client));
            let entry = McpToolEntry::new(client, t);

            self.tools.insert(name, entry);
        }

        Ok(())
    }

    pub async fn load(&mut self, config: &AdoConfig) -> Result<()> {
        if let Err(e) = self.load_embedded() {
            error!("Error Loading embedded tools ({e})")
        }

        self.load_remote(config).await;

        Ok(())
    }

    pub async fn call<P>(&self, params: P) -> Result<String>
    where
        P: AsRef<McpParams>,
    {
        let entry = match self.tools.get(&params.as_ref().tool_name) {
            Some(v) => v,
            None => return Err(Error::ToolNotFound),
        };

        info!("calling: {}", entry.tool.name);

        let mut client = entry.client.lock().await;

        let result = client.call(params.as_ref()).await?;

        Ok(result)
    }

    pub fn list_tools(&self) -> Vec<McpTool> {
        let mut tools: Vec<McpTool> = Vec::new();

        for (_, entry) in self.tools.iter() {
            tools.push(entry.tool.clone());
        }

        tools
    }
}

#[cfg(test)]
mod tests {
    use log::info;
    use omcp::types::McpParams;

    use crate::{config::loader::AdoConfig, error::Result, logging::logger::setup_logger, mcp::matrix::McpMatrix};

    #[tokio::test]
    async fn test_load_remote() -> Result<()> {
        setup_logger(true)?;

        let _config = AdoConfig::from_cwd()?;

        let _mcp_matrix = McpMatrix::new();

        Ok(())
    }

    #[tokio::test]
    async fn test_load_embedded() -> Result<()> {
        setup_logger(true)?;

        let mut matrix = McpMatrix::new();

        matrix.load_embedded()?;

        let tools = matrix.list_tools();

        for t in tools {
            info!("tool={}", t.name);
        }

        let p = McpParams::new("file_write");

        matrix.call(p).await.unwrap();

        Ok(())
    }
}
