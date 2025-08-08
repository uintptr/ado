use std::time::Duration;

use crate::{
    error::{Error, Result},
    http::req::Http,
};
use log::error;
use serde::Deserialize;

//
// https://webd.is/#more
// https://github.com/nicolasff/webdis
//
#[derive(Debug)]
pub struct PersistentStorage {
    user_id: String,
    url: String,
    client: Http,
}

#[derive(Deserialize)]
struct WebdisData {
    #[serde(rename = "GET")]
    get: Option<String>,
}

impl PersistentStorage {
    pub fn new<S, U>(user_id: U, url: S) -> PersistentStorage
    where
        S: AsRef<str>,
        U: AsRef<str>,
    {
        Self {
            user_id: user_id.as_ref().to_string(),
            url: url.as_ref().to_string(),
            client: Http::new(),
        }
    }

    fn build_key<K>(&self, realm: &'static str, user_key: K) -> String
    where
        K: AsRef<str>,
    {
        let digest = md5::compute(user_key.as_ref());
        format!("{}_{}_{digest:x}", self.user_id, realm)
    }

    pub async fn get<S>(&self, realm: &'static str, user_key: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        let key = self.build_key(realm, user_key);
        let get_url = format!("{}/GET/{key}", self.url);

        match self.client.get(get_url, None).await {
            Ok(v) => match v.is_success() {
                true => {
                    let data_string = String::from_utf8(v.data.to_vec())?;
                    let data: WebdisData = serde_json::from_str(&data_string)?;

                    match data.get {
                        Some(v) => Ok(v),
                        None => Err(Error::NotFound),
                    }
                }
                false => Err(Error::StorageWriteFailure),
            },
            Err(e) => {
                error!("{e}");
                Err(e)
            }
        }
    }

    pub async fn set<K, V>(&self, realm: &'static str, user_key: K, value: V, ttl: Duration) -> Result<()>
    where
        K: AsRef<str>,
        V: AsRef<[u8]>,
    {
        let data: Vec<u8> = value.as_ref().to_vec();

        let key = self.build_key(realm, user_key);

        let set_url = match ttl.as_secs() {
            0 => format!("{}/SET/{}", self.url, key),
            ttl_sec => format!("{}/SETEX/{}/{}", self.url, key, ttl_sec),
        };

        match self.client.put(set_url, None, data).await {
            Ok(v) => match v.is_success() {
                true => Ok(()),
                false => Err(Error::StorageWriteFailure),
            },
            Err(e) => {
                error!("{e}");
                Err(e)
            }
        }
    }

    pub async fn del<S>(&self, realm: &'static str, user_key: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        let key = self.build_key(realm, user_key);
        let del_url = format!("{}/DEL/{}", self.url, key);

        match self.client.get(del_url, None).await {
            Ok(v) => match v.is_success() {
                true => Ok(()),
                false => Err(Error::StorageWriteFailure),
            },
            Err(e) => {
                error!("{e}");
                Err(e)
            }
        }
    }
}
#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_persistence() {
        /*
        setup_logger(true).unwrap();

        let user_id = format!("{}", Uuid::new_v4());

        let store = PersistentStorage::new(user_id, "http://127.0.0.1:7379");

        let ret: Result<String> = store.get("bleh").await;
        assert!(ret.is_err());

        let ret = store.set("bleh", "hello").await;
        assert!(ret.is_ok());

        let ret: Result<String> = store.get("bleh").await;
        assert!(ret.is_ok());

        let ret = store.del("bleh").await;
        assert!(ret.is_ok());
        */
    }
}
