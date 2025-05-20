use std::{collections::HashMap, fs, io::Write, path::PathBuf};

use crate::error::{Error, Result};

use super::function_handler::FunctionTrait;

pub struct FunctionWriteFile {
    path: PathBuf,
    content: Vec<u8>,
}

impl FunctionWriteFile {
    pub fn from_args(args: &HashMap<String, String>) -> Result<Self> {
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

        Ok(Self {
            path: PathBuf::new().join(file_name),
            content: file_data.as_bytes().to_vec(),
        })
    }
}

impl FunctionTrait for FunctionWriteFile {
    fn exec(&self) -> Result<String> {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.path)?;

        f.write_all(&self.content)?;

        Ok(format!("successfully wrote {}", self.path.display()))
    }
}
