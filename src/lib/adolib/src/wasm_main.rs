use std::time::Duration;

use crate::{config::file::ConfigFile, error::Result, llm::openai::query::OpenAI, ui::ux::Console};
use tokio::time::sleep;
use wasm_bindgen::prelude::wasm_bindgen;
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

async fn ado_main_async(mut console: Console) -> Result<()> {
    let config = match ConfigFile::load_with_url(CONFIG_URL).await {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("unable to load config file {e}");
            log(&msg);
            return Err(e);
        }
    };

    log(&format!("config: {:?}", config));

    let mut o = OpenAI::new(&config)?;

    log("entering ask()");
    let ret = o.ask(&mut console, None).await;
    log("ask() returned");

    //ret
    Ok(())
}

#[wasm_bindgen]
pub fn ado_main(console: Console) -> i32 {
    wasm_bindgen_futures::spawn_local(async {
        log("hello from ado_main()");
        let ret = ado_main_async(console).await;
        log(&format!("ado_loop() returned {ret:?}"));
    });
    0 // or a placeholder, because async result isn't available synchronously
}
