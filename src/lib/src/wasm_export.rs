use crate::config::file::ConfigFile;
use wasm_bindgen::prelude::wasm_bindgen;
const CONFIG_URL: &str = "http://10.0.0.2/ado.toml";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn query_loop() {
    log("Hello World");

    let config = match ConfigFile::load_with_url(CONFIG_URL) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("unable to load config file {e}");
            log(&msg);
            return;
        }
    };

    log(&format!("config: {:?}", config));
}
