use rust_embed::Embed;

#[derive(Embed)]
#[folder = "config/functions"]
pub struct FunctionAssets;

#[derive(Embed)]
#[folder = "config/whois"]
pub struct WhoisAssets;
