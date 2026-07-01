use std::{
    env, fs,
    path::{Path, PathBuf},
};

use log::{error, info};
use md5::{Digest, Md5};
use sled::Db;

use crate::{
    const_vars::LIB_NAME,
    error::{Error, Result},
};

#[derive(Debug)]
pub struct KVCache {
    db: Db,
}

fn hash_str(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    let hash = hasher.finalize();
    hex::encode(hash)
}

fn get_data_path() -> Result<PathBuf> {
    let data_dir = dirs::data_dir().ok_or_else(|| Error::ConfigNotFound)?;

    let data_dir = data_dir.join(LIB_NAME);

    if !data_dir.exists()
        && let Err(e) = fs::create_dir_all(&data_dir)
    {
        error!("Unanble to create {}", data_dir.display());
        return Err(e.into());
    }

    Ok(data_dir)
}

impl KVCache {
    pub fn new<P>(file_path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        info!("Using {} as cache", file_path.as_ref().display());

        let db = match sled::open(&file_path) {
            Ok(v) => v,
            Err(e) => {
                error!("Unable to open cache @ {}", file_path.as_ref().display());
                return Err(e.into());
            }
        };

        Ok(Self { db })
    }

    pub fn default_path() -> Result<Self> {
        let data_dir = if let Ok(env_dir) = env::var("ADO_CACHE_DIRECTORY") {
            PathBuf::from(env_dir)
        } else {
            get_data_path()?
        };

        info!("data dir: {}", data_dir.display());

        if !data_dir.exists()
            && let Err(e) = fs::create_dir_all(&data_dir)
        {
            error!("Unanble to create {}", data_dir.display());
            return Err(e.into());
        }

        let db_file = data_dir.join("cache.sled");

        KVCache::new(db_file)
    }

    pub fn add<R, K, V>(&self, realm: R, key: K, value: V) -> Result<()>
    where
        R: AsRef<str>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let key = format!("{}_{}", realm.as_ref(), hash_str(key.as_ref().as_bytes()));

        self.db.insert(key, value.as_ref().as_bytes())?;
        Ok(())
    }

    pub fn get_string<R, K>(&self, realm: R, key: K) -> Option<String>
    where
        R: AsRef<str>,
        K: AsRef<str>,
    {
        let key = format!("{}_{}", realm.as_ref(), hash_str(key.as_ref().as_bytes()));
        let Ok(ret) = self.db.get(key) else {
            return None;
        };

        let data = ret?;

        let Ok(data) = String::from_utf8(data.to_vec()) else {
            error!("Unable to convert usafe data to a string");
            return None;
        };

        Some(data)
    }
}
