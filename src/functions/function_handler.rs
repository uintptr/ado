use std::collections::HashMap;

use log::{error, info};

use crate::error::{Error, Result};

use super::{function_files::FunctionFiles, function_whois::FunctionWhois};

pub struct FunctionHandler {
    whois: FunctionWhois,
    files: FunctionFiles,
}

pub fn get_arg<'a>(args: &'a HashMap<String, String>, name: &str) -> Result<&'a String> {
    let value = match args.get(name) {
        Some(v) => v,
        None => {
            error!("{name} was not found in args");
            return Err(Error::MissingArgument {
                name: name.to_string(),
            });
        }
    };

    Ok(value)
}

impl FunctionHandler {
    pub fn new() -> Result<FunctionHandler> {
        let whois = FunctionWhois::new()?;
        let files = FunctionFiles::new();

        Ok(FunctionHandler { whois, files })
    }

    pub fn call(&self, name: &str, args: &HashMap<String, String>) -> Result<String> {
        info!("executing {name}");

        match name {
            "whois_query" => self.whois.query(args),
            "file_write" => self.files.write(args),
            "file_read" => self.files.read(args),
            "file_find" => self.files.find(args),
            _ => {
                error!("function {name} was not found");

                Err(Error::FunctionNotImplemented {
                    name: name.to_string(),
                })
            }
        }
    }
}
