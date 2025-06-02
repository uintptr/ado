use std::time::Duration;

use crate::{
    config::file::ConfigFile,
    error::{Error, Result},
    llm::openai::chain::AIChain,
};
use tokio::time::sleep;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
const CONFIG_URL: &str = "http://10.0.0.2/ado.toml";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

pub async fn log_n_sleep(s: &str) {
    log(s);
    sleep(Duration::from_millis(500)).await
}

async fn ado_load_config() -> Result<()> {
    let config = match ConfigFile::load_with_url(CONFIG_URL).await {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("unable to load config file {e}");
            log(&msg);
            return Err(e);
        }
    };

    log(&format!("config: {:?}", config));

    //ret
    Ok(())
}

#[wasm_bindgen]
pub fn ado_main() -> i32 {
    wasm_bindgen_futures::spawn_local(async {
        let ret = ado_load_config().await;
        log(&format!("ado_loop() returned {ret:?}"));
    });
    0 // or a placeholder, because async result isn't available synchronously
}

#[wasm_bindgen]
pub struct WasmContext {
    config: Option<ConfigFile>,
    chain: Option<AIChain>,
}

// or for your custom error type:
impl From<Error> for JsValue {
    fn from(error: Error) -> Self {
        JsValue::from_str(&error.to_string())
    }
}

#[wasm_bindgen]
impl WasmContext {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmContext {
        WasmContext {
            config: None,
            chain: None,
        }
    }

    pub async fn init(&mut self) -> Result<()> {
        let config = match ConfigFile::load_with_url(CONFIG_URL).await {
            Ok(v) => v,
            Err(e) => {
                let msg = format!("unable to load config file {e}");
                log(&msg);
                return Err(e);
            }
        };

        let chain = AIChain::new(&config);

        self.config = Some(config);

        Ok(())
    }
}
