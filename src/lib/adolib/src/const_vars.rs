use std::time::Duration;

pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const VERGEN_BUILD_DATE: &str = env!("VERGEN_BUILD_DATE");
pub const VERGEN_RUSTC_COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");

pub const CONFIG_FILE_NAME: &str = "config.toml";

pub const CACHE_05_DAYS: Duration = Duration::from_secs(5 * 24 * 60 * 60);
pub const CACHE_30_DAYS: Duration = Duration::from_secs(30 * 24 * 60 * 60);
pub const STORE_PERMANENT: Duration = Duration::from_secs(0);

pub const DIRS_QUALIFIER: &str = "org";
pub const DIRS_ORG: &str = "acme";
pub const DIRS_APP: &str = "ado";
