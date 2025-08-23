use std::collections::HashMap;

use log::{error, info};

use crate::{
    config::loader::AdoConfig,
    data::types::AdoData,
    error::{Error, Result},
    tools::{function_args::ToolArgs, ip_address::FunctionsIp},
};

use super::{
    browser::functions::FunctionsBrowser, files::FunctionsFiles, http::FunctionsHttp, search::FunctionsSearch,
    shell::FunctionsShell, whois::functions::FunctionsWhois,
};

pub struct ToolHandler {
    files: FunctionsFiles,
    http: FunctionsHttp,
    browser: Option<FunctionsBrowser>,
    search: FunctionsSearch,
    shell: FunctionsShell,
    whois: FunctionsWhois,
    ip: FunctionsIp,
}

impl ToolHandler {
    pub fn new(config: &AdoConfig) -> Result<Self> {
        Ok(Self {
            files: FunctionsFiles::new(),
            http: FunctionsHttp::new(),
            browser: FunctionsBrowser::new().ok(),
            search: FunctionsSearch::new(config)?,
            shell: FunctionsShell::new(),
            whois: FunctionsWhois::new()?,
            ip: FunctionsIp::new(),
        })
    }

    pub async fn call<N>(&self, name: N, args: Option<&HashMap<String, String>>) -> Result<AdoData>
    where
        N: AsRef<str>,
    {
        info!("executing {}", name.as_ref());

        let args = ToolArgs::new(args);

        match name.as_ref() {
            "browse" => match &self.browser {
                Some(browser) => browser.browse(&args),
                None => Err(Error::FunctionNotSupported),
            },
            "get_ip_address" => self.ip.get().await,
            "http_get" => self.http.get(&args).await,
            "http_post" => self.http.post(&args).await,
            "file_find" => self.files.find(&args),
            "file_read" => self.files.read(&args),
            "file_list" => self.files.list(&args),
            "file_write" => self.files.write(&args),
            "search" => self.search.search(&args).await,
            "shell_exec" => self.shell.shell_exec(&args),
            "whois_query" => self.whois.query(&args),
            _ => {
                error!("function {} was not found", name.as_ref());

                Err(Error::FunctionNotImplemented {
                    name: name.as_ref().to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::logging::logger::setup_logger;

    use super::ToolArgs;

    const TEST_ARG: &str = "{\"url\":\"http://localhost:8000\",\"http_headers\":[{\"value\":\"Bearer AAABBCCC\",\"key\":\"Authorization\"}]}";

    #[test]
    fn args_test() {
        setup_logger(true).unwrap();

        let args: HashMap<String, String> = serde_json::from_str(TEST_ARG).unwrap();

        let args = ToolArgs::new(Some(&args));

        args.get_string("url").unwrap();
        let _l = args.get_kv_list("http_headers").unwrap();
    }
}
