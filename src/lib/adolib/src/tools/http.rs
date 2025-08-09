use crate::{data::types::AdoData, error::Result, http::req::Http};
use log::info;

use super::function_args::FunctionArgs;

pub struct FunctionsHttp {
    http: Http,
}

impl FunctionsHttp {
    pub fn new() -> FunctionsHttp {
        FunctionsHttp { http: Http::new() }
    }

    pub async fn get(&self, args: &FunctionArgs) -> Result<AdoData> {
        let url = args.get_string("url")?;

        //
        // optional
        //
        let list = args.get_kv_list("http_headers").ok();
        let headers_opt = list.as_ref().map(|v| args.kv_list_to_map(v));

        info!("GET url={url}");
        if let Some(headers) = &headers_opt {
            for (k, v) in headers {
                info!("header={k}:{v}")
            }
        }

        let res = self.http.get(url, headers_opt).await?;

        Ok(AdoData::Http(res))
    }

    pub async fn post(&self, args: &FunctionArgs) -> Result<AdoData> {
        let url = args.get_string("url")?;
        let list = args.get_kv_list("http_headers").ok();
        let headers_opt = list.as_ref().map(|v| args.kv_list_to_map(v));

        info!("POST url={url}");
        if let Some(headers) = &headers_opt {
            for (k, v) in headers {
                info!("header={k}:{v}")
            }
        }

        let res = self.http.post(url, headers_opt).await?;

        Ok(AdoData::Http(res))
    }
}
