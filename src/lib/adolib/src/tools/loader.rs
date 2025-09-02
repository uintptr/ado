use derive_more::Debug;
use log::{error, info};
use omcp::types::McpTool;
use serde::{Deserialize, Serialize};

use crate::{
    error::Result,
    tools::assets::{FunctionAssets, FunctionAssetsPlatform},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tools {
    pub list: Vec<McpTool>,
}

impl Tools {
    pub fn load() -> Result<Self> {
        let mut list = Vec::new();

        info!("loading function assets");

        for name in FunctionAssets::iter() {
            info!("loading {name}");

            let f = match FunctionAssets::get(&name) {
                Some(v) => v,
                None => {
                    error!("unable to read {name}");
                    continue;
                }
            };

            let content = String::from_utf8_lossy(&f.data);

            let inner_list: Vec<McpTool> = serde_json::from_str(&content)?;

            list.extend(inner_list);
        }

        for name in FunctionAssetsPlatform::iter() {
            info!("loading {name}");

            let f = match FunctionAssetsPlatform::get(&name) {
                Some(v) => v,
                None => {
                    error!("unable to read {name}");
                    continue;
                }
            };

            let content = String::from_utf8_lossy(&f.data);

            let inner_list: Vec<McpTool> = serde_json::from_str(&content)?;

            list.extend(inner_list);
        }

        //
        // load platform specific functions (wasm vs console)
        //
        info!("function count: {}", list.len());

        Ok(Self { list })
    }
}
