use std::collections::HashMap;

use crate::error::{Error, Result};

use super::{function_files::FunctionFiles, function_whois::FunctionWhois};

pub struct FunctionHandler {
    whois: FunctionWhois,
    files: FunctionFiles,
}

impl FunctionHandler {
    pub fn new() -> Result<FunctionHandler> {
        let whois = FunctionWhois::new()?;
        let files = FunctionFiles::new();

        Ok(FunctionHandler { whois, files })
    }

    pub fn call(&self, name: &str, args: &HashMap<String, String>) -> Result<String> {
        match name {
            "whois_exists" => self.whois.exists(args),
            "write_file" => self.files.write(args),
            _ => Err(Error::UnknownFunction {
                name: name.to_string(),
            }),
        }
    }
}
