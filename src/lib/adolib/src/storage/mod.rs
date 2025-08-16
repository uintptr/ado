use std::time::Duration;

use async_trait::async_trait;

use crate::error::Result;

#[async_trait(?Send)]
pub trait PersistentStorageTrait {
    async fn get<S>(&self, realm: &'static str, user_key: S) -> Result<String>
    where
        S: AsRef<str>;

    async fn set<K, V>(&self, realm: &'static str, user_key: K, value: V, ttl: Duration) -> Result<()>
    where
        K: AsRef<str>,
        V: AsRef<[u8]>;

    async fn del<S>(&self, realm: &'static str, user_key: S) -> Result<()>
    where
        S: AsRef<str>;
}

#[cfg(target_arch = "wasm32")]
mod webdis;

#[cfg(not(target_arch = "wasm32"))]
mod local;

pub mod persistent {
    #[cfg(not(target_arch = "wasm32"))]
    pub use super::local::LocalStorage as PersistentStorage;
    #[cfg(target_arch = "wasm32")]
    pub use super::webdis::WebdisStorage as PersistentStorage;
}
