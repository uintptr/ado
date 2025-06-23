use reqwest::Client;
use serde_json::Value;

use crate::{
    data::AdoData,
    error::{Error, Result},
};

pub struct FunctionsIp {}

const API_URL: &str = "https://api.ipify.org?format=json";

impl FunctionsIp {
    pub fn new() -> FunctionsIp {
        FunctionsIp {}
    }

    pub async fn get(&self) -> Result<AdoData> {
        let client = Client::new();

        let res = client.get(API_URL).send().await?;

        if !res.status().is_success() {
            return Err(Error::ApiFailure {
                message: res.status().as_str().to_string(),
            });
        }

        let data = res.text().await?;

        let json_data: Value = serde_json::from_str(&data)?;

        let ip_str = match json_data.get("ip") {
            Some(v) => v.as_str().unwrap_or("unknown"),
            None => "unknown",
        };

        Ok(AdoData::String(ip_str.to_string()))
    }
}

#[cfg(test)]
mod tests {

    use crate::logging::logger::setup_logger;

    use super::*;

    #[tokio::test]
    async fn test_get_ip() {
        setup_logger(true).unwrap();
        let ip = FunctionsIp::new();

        let _my_ip = ip.get().await.unwrap();
    }
}
