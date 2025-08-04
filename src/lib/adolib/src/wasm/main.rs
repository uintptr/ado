use std::time::Duration;

use crate::{
    config::loader::ConfigFile,
    data::AdoData,
    error::{Error, Result},
    llm::{openai::chain::AIChain, question::question_detection},
    logging::logger::setup_logger,
    search::google::GoogleCSE,
    storage::webdis::PersistentStorage,
    ui::commands::UserCommands,
    wasm::reddit::RedditQuery,
};
use gloo_utils::format::JsValueSerdeExt;
use log::info;
use tokio::time::sleep;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

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
    chain: AIChain,
    reddit: RedditQuery,
    search: GoogleCSE,
    cache: PersistentStorage,
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

#[wasm_bindgen]
impl AdoWasm {
    //
    // can't make this function async and it also can't return a Result<>.
    //
    // that's why we have to use .unwrap()s and the java script code is now
    // responsible for querying the config from the webdis server
    //
    #[wasm_bindgen(constructor)]
    pub fn new(user_id: &str, storage_url: &str, config: &str) -> AdoWasm {
        setup_logger(true).unwrap();

        console_error_panic_hook::set_once();

        let config = ConfigFile::from_string(config).unwrap();
        let chain = AIChain::new(&config).unwrap();
        let reddit = RedditQuery::new(&config).unwrap();
        let commands = UserCommands::new(&config).unwrap();
        let search = GoogleCSE::new(&config).unwrap();
        let cache = PersistentStorage::new(user_id, storage_url);

        AdoWasm {
            commands,
            chain,
            reddit,
            search,
            cache,
        }
    }

    pub async fn find_sub_reddit(&self, description: &str) -> Result<String> {
        let sub_reddit = match self.cache.get("sub_reddit", description).await {
            Ok(v) => {
                info!("cached {description} -> {v}");
                v
            }
            Err(_) => {
                let data = self.reddit.find_sub(description).await?;
                let _ = self.cache.set("sub_reddit", description, &data);
                data
            }
        };

        Ok(sub_reddit)
    }

    pub async fn query(&mut self, content: &str) -> Result<JsValue> {
        let data = match self.cache.get("query", content).await {
            Ok(v) => serde_json::from_str::<AdoData>(&v)?,
            Err(_) => {
                let data = self.commands.handler(content).await?;

                if let Ok(data_json) = serde_json::to_string(&data) {
                    let _ = self.cache.set("query", content, &data_json).await;
                }
                data
            }
        };

        let obj = JsValue::from_serde(&data)?;

        Ok(obj)
    }

    pub async fn search(&self, query: &str) -> Result<String> {
        let search_data = match self.cache.get("search", query).await {
            Ok(v) => v,
            Err(_) => {
                let data = self.search.query(query).await?;
                let _ = self.cache.set("search", query, &data).await;
                data
            }
        };

        Ok(search_data)
    }

    pub async fn lucky(&self, query: &str) -> Result<String> {
        let lucky_data = match self.cache.get("lucky", query).await {
            Ok(v) => v,
            Err(_) => {
                let data = self.search.lucky(query).await?;
                let _ = self.cache.set("lucky", query, &data).await;
                data
            }
        };

        Ok(lucky_data)
    }

    pub fn is_question(&self, query: &str) -> bool {
        question_detection(query)
    }

    pub fn reset(&mut self) {
        self.chain.reset();
    }
}
