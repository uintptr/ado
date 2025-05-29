use crate::{config::file::ConfigFile, error::Result};
use wasm_bindgen::prelude::wasm_bindgen;
const CONFIG_URL: &str = "http://10.0.0.2/ado.toml";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

async fn ado_loop() -> Result<()> {
    log("Hello World");

    let config = match ConfigFile::load_with_url(CONFIG_URL).await {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("unable to load config file {e}");
            log(&msg);
            return Err(e);
        }
    };

    log(&format!("config: {:?}", config));

    Ok(())
}

#[wasm_bindgen]
pub fn ado_export() -> i32 {
    wasm_bindgen_futures::spawn_local(async {
        let ret = ado_loop().await;
        log(&format!("ado_loop() returned {ret:?}"));
    });
    0 // or a placeholder, because async result isn't available synchronously
}
