use log::{error, info};

use crate::{
    error::{Error, Result},
    functions::function_args::FunctionArgs,
};

use super::{
    browser::browser::FunctionsBrowser, functions_files::FunctionsFiles,
    functions_http::FunctionsHttp, functions_search::FunctionsSearch,
    functions_shell::FunctionsShell, whois::whois::FunctionsWhois,
};

pub struct FunctionHandler {
    files: FunctionsFiles,
    http: FunctionsHttp,
    browser: FunctionsBrowser,
    search: FunctionsSearch,
    shell: FunctionsShell,
    whois: FunctionsWhois,
}

impl FunctionHandler {
    pub fn new() -> Result<FunctionHandler> {
        Ok(FunctionHandler {
            files: FunctionsFiles::new(),
            http: FunctionsHttp::new(),
            browser: FunctionsBrowser::new()?,
            search: FunctionsSearch::new()?,
            shell: FunctionsShell::new(),
            whois: FunctionsWhois::new()?,
        })
    }

    pub fn call(&self, name: &str, args: &str) -> Result<String> {
        info!("executing {name}");

        let args = FunctionArgs::new(args)?;

        match name {
            "browse" => self.browser.browse(&args),
            "http_get" => self.http.get(&args),
            "file_find" => self.files.find(&args),
            "file_read" => self.files.read(&args),
            "file_list" => self.files.list(&args),
            "file_write" => self.files.write(&args),
            "search" => self.search.search(&args),
            "shell_exec" => self.shell.shell_exec(&args),
            "whois_query" => self.whois.query(&args),
            _ => {
                error!("function {name} was not found");

                Err(Error::FunctionNotImplemented {
                    name: name.to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::staples::setup_logger;

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
