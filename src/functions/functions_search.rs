use log::info;

use crate::{
    config::file::{ConfigFile, GoogleConfig},
    error::Result,
};

use super::function_args::FunctionArgs;

pub struct FunctionsSearch {
    google: GoogleConfig,
}

impl FunctionsSearch {
    pub fn new() -> Result<Self> {
        let config = ConfigFile::load()?;
        let google = config.search()?;

        Ok(Self { google })
    }

    pub fn search_query(&self, query: &str) -> Result<String> {
        let res = minreq::get(&self.google.url)
            .with_param("key", &self.google.key)
            .with_param("cx", &self.google.cx)
            .with_param("q", query)
            .with_param("gl", &self.google.geo)
            .send()?;

        let result_string = res.as_str()?;

        Ok(result_string.to_string())
    }

    pub fn search(&self, args: &FunctionArgs) -> Result<String> {
        let query = args.get_string("query")?;

        info!("search term: {query}");

        let data = self.search_query(query)?;

        args.to_base64_string(&data.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use crate::staples::setup_logger;

    use super::FunctionsSearch;

    #[test]
    fn search_test() {
        setup_logger(true).unwrap();
        let search = FunctionsSearch::new().unwrap();

        search.search_query("Hello World").unwrap();
    }
}
