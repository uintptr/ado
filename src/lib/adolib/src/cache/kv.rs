use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use log::{error, info, warn};
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use sled::Db;

use crate::{
    const_vars::LIB_NAME,
    error::{Error, Result},
};

#[derive(Serialize, Deserialize)]
enum KVEntryType {
    String(String),
}

#[derive(Serialize, Deserialize)]
struct KVEntry {
    expiry: u64,
    data: KVEntryType,
}

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

    pub fn add_string<R, K, V>(&self, realm: R, key: K, value: V, expiry: &Duration) -> Result<()>
    where
        R: AsRef<str>,
        K: AsRef<str>,
        V: Into<String>,
    {
        let key = format!("{}_{}", realm.as_ref(), hash_str(key.as_ref().as_bytes()));

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let entry = KVEntry {
            expiry: now + expiry.as_secs(),
            data: KVEntryType::String(value.into()),
        };

        let entry_str = serde_json::to_string(&entry)?;

        self.db.insert(key, entry_str.as_bytes())?;

        Ok(())
    }

    pub fn get_string<R, K>(&self, realm: R, key: K) -> Result<String>
    where
        R: AsRef<str>,
        K: AsRef<str>,
    {
        let key = format!("{}_{}", realm.as_ref(), hash_str(key.as_ref().as_bytes()));
        let Ok(ret) = self.db.get(&key) else {
            return Err(Error::NotFound);
        };

        let Some(data) = ret else {
            return Err(Error::NotFound);
        };

        let data = String::from_utf8(data.to_vec()).map_err(|e| {
            error!("Unable to convert unsafe data to a string");
            e
        })?;

        let entry: KVEntry = serde_json::from_str(&data).map_err(|e| {
            error!("Unable to deserialize {data}");
            e
        })?;

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        if now > entry.expiry {
            warn!("{key} is expired");
            if let Err(e) = self.db.remove(&key) {
                error!("Unable to delete {key} ({e})");
            }
            return Err(Error::Expired);
        }

        let KVEntryType::String(s) = entry.data;

        Ok(s)
    }
}
