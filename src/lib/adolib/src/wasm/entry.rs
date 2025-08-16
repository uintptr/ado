use std::time::Duration;

use crate::{
    config_file::loader::ConfigFile,
    data::types::{AdoData, AdoDataMarkdown},
    error::{Error, Result},
    llm::{provider::LLMChain, question::question_detection},
    logging::logger::setup_logger,
    storage::persistent::PersistentStorage,
    ui::commands::UserCommands,
};
use gloo_utils::format::JsValueSerdeExt;
use serde::Serialize;
use tokio::time::sleep;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::window;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

pub async fn log_n_sleep(s: &str) {
    log(s);
    sleep(Duration::from_millis(500)).await
}

#[wasm_bindgen]
pub struct AdoWasm {
    commands: UserCommands,
    chain: LLMChain,
}

// or for your custom error type:
impl From<Error> for JsValue {
    fn from(error: Error) -> Self {
        JsValue::from_str(&error.to_string())
    }
}

#[wasm_bindgen]
pub struct AdoWasmCommand {
    name: String,
    short: String,
    desc: String,
}

#[derive(Serialize)]
pub struct AdoWasmQueryResponse {
    pub data: AdoData,
    pub markdown: String,
}

#[wasm_bindgen]
impl AdoWasmCommand {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn short(&self) -> String {
        self.short.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn desc(&self) -> String {
        self.desc.clone()
    }
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

        let config = ConfigFile::from_string(config).unwrap();
        let chain = LLMChain::new(&config).unwrap();
        let cache = PersistentStorage::new(user_id, storage_url);
        let commands = UserCommands::new(&config, cache).unwrap();

        AdoWasm { commands, chain }
    }

    pub async fn query(&mut self, content: &str) -> Result<JsValue> {
        let data = self.commands.handler(content).await?;

        let resp = AdoWasmQueryResponse {
            data: data.clone(),
            markdown: data.to_markdown()?,
        };

        let obj = JsValue::from_serde(&resp)?;

        Ok(obj)
    }

    pub fn is_question(&self, query: &str) -> bool {
        question_detection(query)
    }

    pub fn reset(&mut self) {
        self.chain.reset();
    }
}
