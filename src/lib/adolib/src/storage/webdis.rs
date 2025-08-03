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

    fn build_user_key<S>(&self, key: S) -> String
    where
        S: AsRef<str>,
    {
        let hex_key_string = format!("{}{}", self.user_id, key.as_ref());
        let digest = md5::compute(hex_key_string);
        format!("{digest:x}")
    }

    pub async fn get_raw<S>(&self, key: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        let get_url = format!("{}/GET/{}", self.url, key.as_ref());

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

    pub async fn get<S>(&self, key: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        let user_key = self.build_user_key(key);

        self.get_raw(user_key).await
    }

    pub async fn set<K, V>(&self, key: K, value: V) -> Result<()>
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let data: Vec<u8> = value.as_ref().into();

        let user_key = self.build_user_key(key);
        let set_url = format!("{}/SET/{}", self.url, user_key);

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

    pub async fn del<S>(&self, key: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        let user_key = self.build_user_key(key);
        let del_url = format!("{}/DEL/{}", self.url, user_key);

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

    use uuid::Uuid;

    use crate::{error::Result, logging::logger::setup_logger, storage::webdis::PersistentStorage};

    #[tokio::test]
    async fn test_persistence() {
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
    }
}
