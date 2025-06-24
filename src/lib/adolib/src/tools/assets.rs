use rust_embed::Embed;

#[cfg(target_arch = "wasm32")]
#[derive(Embed)]
#[folder = "config/functions/wasm"]
pub struct FunctionAssetsPlatform;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Embed)]
#[folder = "config/functions/native"]
pub struct FunctionAssetsPlatform;

#[derive(Embed)]
#[folder = "config/functions/all"]
pub struct FunctionAssets;

#[derive(Embed)]
#[folder = "config/whois"]
pub struct WhoisAssets;
