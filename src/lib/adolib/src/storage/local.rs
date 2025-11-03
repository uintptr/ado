use std::{fs, path::Path, time::Duration};

use crate::{
    const_vars::{DIRS_APP, DIRS_ORG, DIRS_QUALIFIER},
    error::{Error, Result},
    storage::PersistentStorageTrait,
};
use async_trait::async_trait;
use directories::ProjectDirs;
use sled::Db;

use log::{error, info};

#[derive(Debug, Clone)]
pub struct LocalStorage {
    tree: Option<Db>,
}

impl LocalStorage {
    pub fn new() -> Result<Self> {
        let dirs = ProjectDirs::from(DIRS_QUALIFIER, DIRS_ORG, DIRS_APP).ok_or(Error::NotFound)?;

        let cache_dir = dirs.cache_dir();

        info!("cache dir {}", cache_dir.display());

        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        LocalStorage::from_path(cache_dir)
    }

    pub fn from_path<P>(file_path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let tree = match sled::open(file_path) {
            Ok(v) => Some(v),
            Err(e) => {
                error!("{e}");
                None
            }
        };

        Ok(Self { tree })
    }

    fn build_key<K>(&self, realm: &'static str, user_key: K) -> String
    where
        K: AsRef<str>,
    {
        let digest = md5::compute(user_key.as_ref());
        format!("{realm}_{digest:x}")
    }
}

#[async_trait(?Send)]
impl PersistentStorageTrait for LocalStorage {
    async fn get<S>(&self, realm: &'static str, user_key: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        match &self.tree {
            Some(tree) => {
                let key = self.build_key(realm, user_key);

                let data = match tree.get(key)? {
                    Some(v) => v,
                    None => {
                        return Err(Error::NotFound);
                    }
                };

                let data_string = String::from_utf8(data.to_vec())?;

                Ok(data_string)
            }
            None => Err(Error::NotInitialized),
        }
    }

    async fn set<K, V>(&self, realm: &'static str, user_key: K, value: V, _ttl: Duration) -> Result<()>
    where
        K: AsRef<str>,
        V: AsRef<[u8]>,
    {
        match &self.tree {
            Some(tree) => {
                let key = self.build_key(realm, user_key);

                let data: Vec<u8> = value.as_ref().to_vec();

                tree.insert(key, data)?;

                Ok(())
            }
            None => Err(Error::NotInitialized),
        }
    }

    async fn del<S>(&self, realm: &'static str, user_key: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        match &self.tree {
            Some(tree) => {
                let key = self.build_key(realm, user_key);

                tree.remove(key)?;

                Ok(())
            }
            None => Err(Error::NotInitialized),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::storage::{PersistentStorageTrait, local::LocalStorage};

    #[tokio::test]
    async fn test_local() {
        crate::logging::logger::setup_logger(true).unwrap();

        let td = tempfile::Builder::new().prefix("ls_test_").tempdir().unwrap();

        let db_file = td.path().join("test.sled");

        let ls = LocalStorage::from_path(db_file).unwrap();

        let ret = ls.get("test", "hello").await;
        assert!(ret.is_err());

        let ret = ls.set("test", "hello", "world", Duration::from_secs(0)).await;
        assert!(ret.is_ok());

        let ret = ls.get("test", "hello").await;
        assert!(ret.is_ok());

        let ret = ls.del("test", "hello").await;
        assert!(ret.is_ok());

        let ret = ls.get("test", "hello").await;
        assert!(ret.is_err());
    }
}
