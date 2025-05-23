use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct FunctionArgsKV {
    pub key: String,
    pub value: String,
}

pub struct FunctionArgs {
    inner: HashMap<String, Value>,
}

impl FunctionArgs {
    pub fn new(args: &str) -> Result<Self> {
        let map: HashMap<String, Value> = serde_json::from_str(args)?;

        Ok(FunctionArgs { inner: map })
    }

    pub fn get_string(&self, key: &str) -> Result<&str> {
        let v = self.inner.get(key).ok_or(Error::MissingArgument {
            name: key.to_string(),
        })?;

        v.as_str().ok_or(Error::TypeError {
            error: format!("{key} is not a string"),
        })
    }

    pub fn get_kv_list(&self, key: &str) -> Result<Vec<FunctionArgsKV>> {
        let v = self.inner.get(key).ok_or(Error::MissingArgument {
            name: key.to_string(),
        })?;

        let list: Vec<FunctionArgsKV> = serde_json::from_value(v.clone())?;

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
