use std::{collections::HashMap, path::Path};

use log::info;
use whois_rust::{WhoIs, WhoIsLookupOptions};

use crate::{
    error::{Error, Result},
    staples::find_file,
};

pub struct FunctionWhois {
    provider: WhoIs,
}

impl FunctionWhois {
    pub fn new() -> Result<Self> {
        let rel_servers = Path::new("config").join("whois_servers.json");
        let servers_file = find_file(rel_servers)?;

        let provider = WhoIs::from_path(servers_file)?;

        Ok(Self { provider })
    }

    pub fn exists(&self, args: &HashMap<String, String>) -> Result<String> {
        let domain_name = match args.get("domain_name") {
            Some(v) => v,
            None => {
                return Err(Error::MissingArgument {
                    name: "domain_name".to_string(),
                });
            }
        };

        info!("looking for domain_name={domain_name}");

        let opts = WhoIsLookupOptions::from_str(domain_name)?;

        self.provider.lookup(opts)?;

        Ok(format!("{domain_name} is registered"))
    }
}
