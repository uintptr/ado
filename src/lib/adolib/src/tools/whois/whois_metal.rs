use log::info;
use whois_rust::{WhoIs, WhoIsLookupOptions};

use crate::{
    data::types::AdoData,
    error::{Error, Result},
    tools::{assets::WhoisAssets, function_args::ToolArgs},
};

pub struct FunctionsWhois {
    provider: WhoIs,
}

impl FunctionsWhois {
    pub fn new() -> Result<Self> {
        let config_file = WhoisAssets::get("whois_servers.json").ok_or(Error::FileNotFoundError {
            file_path: "whois_servers.json".into(),
        })?;

        let config_string = String::from_utf8(config_file.data.to_vec())?;

        let provider = WhoIs::from_string(config_string)?;

        Ok(Self { provider })
    }

    pub fn query_domain(&self, domain_name: &str) -> Result<String> {
        info!("looking for domain_name={domain_name}");

        let opts = WhoIsLookupOptions::from_str(domain_name)?;

        let data = self.provider.lookup(opts)?;

        Ok(data)
    }

    pub fn query(&self, args: &ToolArgs) -> Result<AdoData> {
        let domain_name = args.get_string("domain_name")?;
        let res = self.query_domain(domain_name)?;
        Ok(AdoData::String(res))
    }
}

#[cfg(test)]
mod tests {

    use crate::logging::logger::setup_logger;

    use super::*;

    #[test]
    fn whois_test() {
        setup_logger(true).unwrap();
        let whois = FunctionsWhois::new().unwrap();
        whois.query_domain("example.com").unwrap();
    }
}
