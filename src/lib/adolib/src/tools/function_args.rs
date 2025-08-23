use std::collections::HashMap;

use base64::{Engine, prelude::BASE64_STANDARD};

use serde::Deserialize;

use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct FunctionArgsKV {
    pub key: String,
    pub value: String,
}

pub struct ToolArgs<'a> {
    inner: Option<&'a HashMap<String, String>>,
}

impl<'a> ToolArgs<'a> {
    pub fn new(args: Option<&'a HashMap<String, String>>) -> Self {
        Self { inner: args }
    }

    pub fn to_base64_string(&self, data: &[u8]) -> Result<String> {
        let encoded_data = BASE64_STANDARD.encode(data);

        let mut msg = "data:text/plain;charset=utf-8;base64,".to_string();

        msg.push_str(&encoded_data);

        Ok(msg)
    }

    pub fn get_string(&self, key: &str) -> Result<&str> {
        match self.inner {
            Some(v) => {
                let value = v.get(key).ok_or(Error::MissingArgument { name: key.to_string() })?;
                Ok(value.as_str())
            }
            None => Err(Error::MissingArgument { name: key.to_string() }),
        }
    }

    pub fn get_kv_list(&self, key: &str) -> Result<Vec<FunctionArgsKV>> {
        match self.inner {
            Some(v) => {
                let value = v.get(key).ok_or(Error::MissingArgument { name: key.to_string() })?;
                let list: Vec<FunctionArgsKV> = serde_json::from_str(value.as_str())?;
                Ok(list)
            }
            None => Err(Error::MissingArgument { name: key.to_string() }),
        }
    }

    pub fn kv_list_to_map<'b>(&self, list: &'b [FunctionArgsKV]) -> HashMap<&'b str, &'b str> {
        let mut map: HashMap<&str, &str> = HashMap::new();

        for i in list.iter() {
            map.insert(&i.key, &i.value);
        }

        map
    }
}
