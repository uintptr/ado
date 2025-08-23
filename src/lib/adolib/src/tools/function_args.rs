use std::collections::HashMap;

use base64::{Engine, prelude::BASE64_STANDARD};

use serde::Deserialize;

use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct FunctionArgsKV {
    pub key: String,
    pub value: String,
}

pub struct ToolArgs {
    inner: HashMap<String, String>,
}

impl ToolArgs {
    pub fn new(args: Option<&HashMap<String, String>>) -> Self {
        let inner = match args {
            Some(v) => v.clone(),
            None => HashMap::new(),
        };
        Self { inner }
    }

    pub fn to_base64_string(&self, data: &[u8]) -> Result<String> {
        let encoded_data = BASE64_STANDARD.encode(data);

        let mut msg = "data:text/plain;charset=utf-8;base64,".to_string();

        msg.push_str(&encoded_data);

        Ok(msg)
    }

    pub fn get_string(&self, key: &str) -> Result<&str> {
        let v = self.inner.get(key).ok_or(Error::MissingArgument { name: key.to_string() })?;
        Ok(v.as_str())
    }

    pub fn get_kv_list(&self, key: &str) -> Result<Vec<FunctionArgsKV>> {
        let v = self.inner.get(key).ok_or(Error::MissingArgument { name: key.to_string() })?;
        let list: Vec<FunctionArgsKV> = serde_json::from_str(v.as_str())?;
        Ok(list)
    }

    pub fn kv_list_to_map<'a>(&self, list: &'a [FunctionArgsKV]) -> HashMap<&'a str, &'a str> {
        let mut map: HashMap<&str, &str> = HashMap::new();

        for i in list.iter() {
            map.insert(&i.key, &i.value);
        }

        map
    }
}
