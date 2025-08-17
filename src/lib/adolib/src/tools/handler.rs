use log::{error, info};

use crate::{
    config::loader::AdoConfig,
    data::types::AdoData,
    error::{Error, Result},
    tools::{function_args::FunctionArgs, ip_address::FunctionsIp},
};

use super::{
    browser::functions::FunctionsBrowser, files::FunctionsFiles, http::FunctionsHttp, search::FunctionsSearch,
    shell::FunctionsShell, whois::functions::FunctionsWhois,
};

pub struct FunctionHandler {
    files: FunctionsFiles,
    http: FunctionsHttp,
    browser: Option<FunctionsBrowser>,
    search: FunctionsSearch,
    shell: FunctionsShell,
    whois: FunctionsWhois,
    ip: FunctionsIp,
}

impl FunctionHandler {
    pub fn new(config: &AdoConfig) -> Result<FunctionHandler> {
        Ok(FunctionHandler {
            files: FunctionsFiles::new(),
            http: FunctionsHttp::new(),
            browser: FunctionsBrowser::new().ok(),
            search: FunctionsSearch::new(config)?,
            shell: FunctionsShell::new(),
            whois: FunctionsWhois::new()?,
            ip: FunctionsIp::new(),
        })
    }

    pub async fn call(&self, name: &str, args: &str) -> Result<AdoData> {
        info!("executing {name}");

        let args = FunctionArgs::new(args)?;

        match name {
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
                error!("function {name} was not found");

                Err(Error::FunctionNotImplemented { name: name.to_string() })
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::logging::logger::setup_logger;

    use super::FunctionArgs;

    const TEST_ARG: &str = "{\"url\":\"http://localhost:8000\",\"http_headers\":[{\"value\":\"Bearer AAABBCCC\",\"key\":\"Authorization\"}]}";

    #[test]
    fn args_test() {
        setup_logger(true).unwrap();

        let args = FunctionArgs::new(TEST_ARG).unwrap();

        args.get_string("url").unwrap();
        let _l = args.get_kv_list("http_headers").unwrap();
    }
}
