use std::{collections::HashMap, path::Path};

use whois_rust::{WhoIs, WhoIsLookupOptions};

use crate::{
    error::{Error, Result},
    staples::find_file,
};

use log::error;

use super::function_handler::FunctionTrait;

pub struct FunctionWhoisExists {
    domain_name: String,
}

impl FunctionWhoisExists {
    pub fn from_args(args: &HashMap<String, String>) -> Result<FunctionWhoisExists> {
        let domain = match args.get("domain_name") {
            Some(v) => v,
            None => {
                return Err(Error::MissingArgument {
                    name: "domain_name".to_string(),
                });
            }
        };

        Ok(FunctionWhoisExists {
            domain_name: domain.into(),
        })
    }
}

impl FunctionTrait for FunctionWhoisExists {
    fn exec(&self) -> Result<String> {
        let rel_servers = Path::new("config").join("whois_servers.json");
        let servers_file = find_file(rel_servers)?;

        let whois = WhoIs::from_path(servers_file)?;

        let opts = WhoIsLookupOptions::from_str(&self.domain_name)?;

        let res_string = match whois.lookup(opts) {
            Ok(_) => format!("{} exists", self.domain_name),
            Err(e) => {
                error!("{e}");
                format!("{} does not exist", self.domain_name)
            }
        };

        Ok(res_string)
    }
}
