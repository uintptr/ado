use std::{collections::HashMap, fs, io::Write};

use crate::error::{Error, Result};

pub struct FunctionFiles;

impl FunctionFiles {
    pub fn new() -> FunctionFiles {
        FunctionFiles {}
    }

    pub fn write(&self, args: &HashMap<String, String>) -> Result<String> {
        let file_name = match args.get("file_name") {
            Some(v) => v,
            None => {
                return Err(Error::MissingArgument {
                    name: "file_name".to_string(),
                });
            }
        };

        let file_data = match args.get("file_data") {
            Some(v) => v,
            None => {
                return Err(Error::MissingArgument {
                    name: "file_data".to_string(),
                });
            }
        };

        let mut f = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(file_name)?;

        f.write_all(file_data.as_bytes())?;

        let msg = format!("{file_name} was successfully written");

        Ok(msg)
    }
}
