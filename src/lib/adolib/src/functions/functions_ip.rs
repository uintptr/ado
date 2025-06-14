use reqwest::Client;
use serde_json::Value;

use crate::error::Result;

pub struct FunctionsIp {}

const API_URL: &str = "https://api.ipify.org?format=json";

impl FunctionsIp {
    pub fn new() -> FunctionsIp {
        FunctionsIp {}
    }

    pub async fn get(&self) -> Result<String> {
        let client = Client::new();

        let res = client.get(API_URL).send().await?;

        if !res.status().is_success() {
            return Ok("API Failure".to_string());
        }

        let data = res.text().await?;

        let json_data: Value = serde_json::from_str(&data)?;

        let ip_str = match json_data.get("ip") {
            Some(v) => v.as_str().unwrap_or("unknown"),
            None => "unknown",
        };

        Ok(ip_str.to_string())
    }
}

#[cfg(test)]
mod tests {

    use log::info;

    use crate::logging::logger::setup_logger;

    use super::*;

    #[tokio::test]
    async fn test_get_ip() {
        setup_logger(true).unwrap();
        let ip = FunctionsIp::new();

        let my_ip = ip.get().await.unwrap();

        info!("ip: {my_ip}")
    }
}
