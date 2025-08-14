use std::time::Duration;

use crate::{
    error::{Error, Result},
    storage::PersistentStorageTrait,
};
use async_trait::async_trait;

#[derive(Debug)]
pub struct LocalStorage {}

impl LocalStorage {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl PersistentStorageTrait for LocalStorage {
    async fn get<S>(&self, _realm: &'static str, _user_key: S) -> Result<String>
    where
        S: AsRef<str> + Send,
    {
        Err(Error::NotFound)
    }

    async fn set<K, V>(&self, _realm: &'static str, _user_key: K, _value: V, _ttl: Duration) -> Result<()>
    where
        K: AsRef<str> + Send,
        V: AsRef<[u8]> + Send,
    {
        Err(Error::NotFound)
    }

    async fn del<S>(&self, _realm: &'static str, _user_key: S) -> Result<()>
    where
        S: AsRef<str> + Send,
    {
        Err(Error::NotFound)
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_local() {}
}
