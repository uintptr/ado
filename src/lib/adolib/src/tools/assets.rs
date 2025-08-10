use rust_embed::Embed;

#[cfg(target_arch = "wasm32")]
#[derive(Embed)]
#[folder = "tools/functions/wasm"]
pub struct FunctionAssetsPlatform;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Embed)]
#[folder = "tools/functions/native"]
pub struct FunctionAssetsPlatform;

#[derive(Embed)]
#[folder = "tools/functions/all"]
pub struct FunctionAssets;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Embed)]
#[folder = "tools/whois"]
pub struct WhoisAssets;
