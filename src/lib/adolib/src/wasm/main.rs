use std::time::Duration;

use crate::{
    config::file::ConfigFile,
    error::{Error, Result},
    llm::{openai::chain::AIChain, question::question_detection},
    logging::logger::setup_logger,
    search::google::GoogleCSE,
    ui::commands::UserCommands,
    wasm::reddit::RedditQuery,
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
pub struct AdoWasm {
    commands: UserCommands,
    chain: AIChain,
    reddit: RedditQuery,
    search: GoogleCSE,
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
    #[wasm_bindgen(constructor)]
    pub fn new(config: &str) -> AdoWasm {
        setup_logger(true).unwrap();

        console_error_panic_hook::set_once();

        let config = ConfigFile::from_string(config).unwrap();
        let chain = AIChain::new(&config).unwrap();
        let reddit = RedditQuery::new(&config).unwrap();
        let commands = UserCommands::new(&config).unwrap();
        let search = GoogleCSE::new(&config).unwrap();

        AdoWasm {
            commands,
            chain,
            reddit,
            search,
        }
    }

    pub async fn find_sub_reddit(&self, description: &str) -> Result<String> {
        self.reddit.find_sub(description).await
    }

    pub async fn query(&mut self, content: &str) -> Result<String> {
        //
        // see if it's a command first
        //
        if content.starts_with("/") {
            let data = self.commands.handler(content).await?;
            let data_string: String = data.try_into()?;
            Ok(format!("```sh\n{data_string}\n```"))
        } else {
            let mut ret_list = Vec::new();

            for d in self.chain.query(content).await? {
                let d: String = d.try_into()?;
                ret_list.push(d);
            }

            Ok(ret_list.join("\n"))
        }
    }

    pub fn usage(&self) -> String {
        format!("```\n{}\n```", self.commands.usage())
    }

    pub async fn search(&self, query: &str) -> Result<String> {
        self.search.query(query).await
    }

    pub async fn lucky(&self, query: &str) -> Result<String> {
        self.search.lucky(query).await
    }

    pub fn is_question(&self, query: &str) -> bool {
        question_detection(query)
    }

    pub fn reset(&mut self) {
        self.chain.reset();
    }
}
