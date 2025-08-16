use std::{
    env::{self, home_dir},
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    const_vars::DOT_DIRECTORY,
    error::{Error, Result},
    storage::PersistentStorageTrait,
};
use async_trait::async_trait;
use sled::Db;

#[derive(Debug)]
pub struct LocalStorage {
    tree: Db,
}

impl LocalStorage {
    pub fn new() -> Result<Self> {
        let home = match home_dir() {
            Some(v) => v,
            None => {
                let home_env = env::var("HOME")?;
                PathBuf::from(home_env)
            }
        };

        let ado_dir = home.join(DOT_DIRECTORY);

        if !ado_dir.exists() {
            fs::create_dir(&ado_dir)?;
        }

        let cache_file = ado_dir.join("cache.db");

        LocalStorage::from_path(cache_file)
    }

    pub fn from_path<P>(file_path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let tree = sled::open(file_path)?;

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
        let key = self.build_key(realm, user_key);

        let data = match self.tree.get(key)? {
            Some(v) => v,
            None => {
                return Err(Error::NotFound);
            }
        };

        let data_string = String::from_utf8(data.to_vec())?;

        Ok(data_string)
    }

    async fn set<K, V>(&self, realm: &'static str, user_key: K, value: V, _ttl: Duration) -> Result<()>
    where
        K: AsRef<str>,
        V: AsRef<[u8]>,
    {
        let key = self.build_key(realm, user_key);

        let data: Vec<u8> = value.as_ref().to_vec();

        self.tree.insert(key, data)?;

        Ok(())
    }

    async fn del<S>(&self, realm: &'static str, user_key: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        let key = self.build_key(realm, user_key);

        self.tree.remove(key)?;

        Ok(())
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
