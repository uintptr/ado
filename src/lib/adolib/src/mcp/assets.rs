use rust_embed::Embed;

#[cfg(target_arch = "wasm32")]
#[derive(Embed)]
#[folder = "tools/functions/wasm"]
pub struct McpAssetsPlatform;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Embed)]
#[folder = "tools/functions/native"]
pub struct McpAssetsPlatform;

#[derive(Embed)]
#[folder = "tools/functions/all"]
pub struct McpAssetsAll;

/*
#[cfg(not(target_arch = "wasm32"))]
#[derive(Embed)]
#[folder = "tools/whois"]
pub struct McpWhoisAssets;
*/
