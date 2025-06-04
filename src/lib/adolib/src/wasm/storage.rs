use redis::{Client, Commands, Connection, FromRedisValue, ToRedisArgs};

use crate::error::Result;

pub struct PersistentStorage {
    client: Client,
    con: Connection,
}

impl PersistentStorage {
    pub fn new(server: &str) -> Result<PersistentStorage> {
        let uri = format!("redis://{server}");

        let client = redis::Client::open(uri)?;
        let con = client.get_connection()?;

        Ok(PersistentStorage { client, con })
    }

    pub fn get<T>(&mut self, key: &str) -> Result<T>
    where
        T: FromRedisValue,
    {
        let data = self.con.get(key)?;
        Ok(data)
    }

    pub fn set<T>(&mut self, key: &str, value: T) -> Result<()>
    where
        T: ToRedisArgs,
    {
        let _: () = self.con.set(key, value)?;
        Ok(())
    }

    pub fn del(&mut self, key: &str) -> Result<()> {
        let _: () = self.con.del(key)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use log::info;

    use crate::{error::Result, staples::setup_logger, storage::persistence::PersistentStorage};

    #[test]
    fn test_persistence() {
        setup_logger(true).unwrap();

        let mut store = PersistentStorage::new("localhost").unwrap();

        let ret: Result<String> = store.get("bleh");
        assert!(ret.is_err());

        let ret = store.set("bleh", "hello");
        assert!(ret.is_ok());

        let ret: Result<String> = store.get("bleh");
        assert!(ret.is_ok());
        info!("--> {}", ret.unwrap());

        let ret = store.del("bleh");
        assert!(ret.is_ok());
    }
}
