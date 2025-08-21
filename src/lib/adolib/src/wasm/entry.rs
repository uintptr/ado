use crate::{
    config::loader::AdoConfig,
    data::types::{AdoData, AdoDataMarkdown},
    error::{Error, Result},
    llm::{chain::LLMChain, question::question_detection},
    logging::logger::setup_logger,
    storage::persistent::PersistentStorage,
    ui::commands::UserCommands,
    wasm::consoles::{WasmAsyncConsole, WasmSyncConsole},
};

use gloo_utils::format::JsValueSerdeExt;
use serde::Serialize;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

use web_sys::window;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
    #[wasm_bindgen(js_name = wasm_display)]
    pub fn wasm_display(s: JsValue);
    #[wasm_bindgen(js_name = wasm_display_spinner_start)]
    pub fn wasm_display_spinner_start();
    #[wasm_bindgen(js_name = wasm_display_spinner_stop)]
    pub fn wasm_display_spinner_stop();
}

#[wasm_bindgen]
pub struct AdoWasm {
    commands: UserCommands,
    chain: LLMChain,
    async_console: WasmAsyncConsole,
    sync_console: WasmSyncConsole,
}

// or for your custom error type:
impl From<Error> for JsValue {
    fn from(error: Error) -> Self {
        JsValue::from_str(&error.to_string())
    }
}

#[derive(Serialize)]
pub struct WasmQueryResponse {
    pub data: AdoData,
    pub markdown: String,
}

fn build_storage_url() -> Result<String> {
    let window = window().ok_or(Error::NotFound)?;

    let hostname = match window.location().hostname() {
        Ok(v) => v,
        Err(_) => return Err(Error::NotFound),
    };

    let proto = match window.location().protocol() {
        Ok(v) => v,
        Err(_) => return Err(Error::NotFound),
    };

    Ok(format!("{proto}{hostname}/webdis"))
}

#[wasm_bindgen]
impl AdoWasm {
    //
    // can't make this function async and it also can't return a Result<>.
    //
    // that's why we have to use .unwrap()s and the java script code is now
    // responsible for querying the config from the webdis server
    //
    #[wasm_bindgen(constructor)]
    pub fn new(user_id: &str, config: &str) -> AdoWasm {
        setup_logger(true).unwrap();

        console_error_panic_hook::set_once();

        let storage_url = build_storage_url().unwrap();

        let config = AdoConfig::from_string(config).unwrap();
        let chain = LLMChain::new(&config).unwrap();
        let cache = PersistentStorage::new(user_id, storage_url);
        let commands = UserCommands::new(&config, cache).unwrap();

        AdoWasm {
            commands,
            chain,
            async_console: WasmAsyncConsole {},
            sync_console: WasmSyncConsole::new(),
        }
    }

    pub async fn query(&mut self, content: &str) -> Result<JsValue> {
        self.sync_console.clear();
        self.commands.handler(content, &mut self.sync_console).await?;

        match self.sync_console.data_list.pop() {
            Some(data) => {
                let resp = WasmQueryResponse {
                    data: data.clone(),
                    markdown: data.to_markdown()?,
                };

                let obj = JsValue::from_serde(&resp)?;

                Ok(obj)
            }
            None => Err(Error::Empty),
        }
    }

    pub async fn queue(&mut self, content: &str) -> Result<()> {
        self.commands.handler(content, &mut self.async_console).await
    }

    pub fn is_question(&self, query: &str) -> bool {
        question_detection(query)
    }

    pub fn reset(&mut self) {
        self.chain.reset();
    }
}
