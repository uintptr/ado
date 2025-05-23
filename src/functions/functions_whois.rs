use std::path::Path;

use log::info;
use whois_rust::{WhoIs, WhoIsLookupOptions};

use crate::{error::Result, staples::find_file};

use super::function_args::FunctionArgs;

pub struct FunctionsWhois {
    provider: WhoIs,
}

impl FunctionsWhois {
    pub fn new() -> Result<Self> {
        let rel_servers = Path::new("config").join("whois_servers.json");
        let servers_file = find_file(rel_servers)?;

        let provider = WhoIs::from_path(servers_file)?;

        Ok(Self { provider })
    }

    pub fn query_domain(&self, domain_name: &str) -> Result<String> {
        info!("looking for domain_name={domain_name}");

        let opts = WhoIsLookupOptions::from_str(domain_name)?;

        let data = self.provider.lookup(opts)?;

        Ok(data)
    }

    pub fn query(&self, args: &FunctionArgs) -> Result<String> {
        let domain_name = args.get_string("domain_name")?;
        self.query_domain(domain_name)
    }
}

#[cfg(test)]
mod tests {

    use crate::staples::setup_logger;

    use super::*;

    #[test]
    fn whois_test() {
        setup_logger(true).unwrap();
        let whois = FunctionsWhois::new().unwrap();
        whois.query_domain("example.com").unwrap();
    }
}
