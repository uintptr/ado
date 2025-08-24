use crate::{
    data::types::{AdoData, AdoDataMarkdown},
    error::Result,
    ui::ConsoleDisplayTrait,
    wasm::entry::{WasmQueryResponse, wasm_display, wasm_display_spinner_start, wasm_display_spinner_stop},
};
use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::JsValue;

pub struct WasmSyncConsole {
    pub data_list: Vec<AdoData>,
}

impl WasmSyncConsole {
    pub fn new() -> Self {
        Self { data_list: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.data_list.clear();
    }
}
impl ConsoleDisplayTrait for WasmSyncConsole {
    fn start_spinner(&mut self) {
        wasm_display_spinner_start();
    }
    fn stop_spinner(&mut self) {
        wasm_display_spinner_stop();
    }
    fn display<D>(&mut self, data: D) -> Result<()>
    where
        D: AsRef<AdoData>,
    {
        self.data_list.push(data.as_ref().clone());
        Ok(())
    }
    fn display_string<S>(&mut self, value: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        self.display(AdoData::String(value.as_ref().into()))
    }
}

pub struct WasmAsyncConsole {}

impl WasmAsyncConsole {}

impl ConsoleDisplayTrait for WasmAsyncConsole {
    fn start_spinner(&mut self) {
        wasm_display_spinner_start();
    }
    fn stop_spinner(&mut self) {
        wasm_display_spinner_stop();
    }
    fn display<D>(&mut self, data: D) -> Result<()>
    where
        D: AsRef<AdoData>,
    {
        let resp = WasmQueryResponse {
            data: data.as_ref(),
            markdown: data.as_ref().to_markdown()?,
        };

        let obj = JsValue::from_serde(&resp)?;
        wasm_display(obj);
        Ok(())
    }
    fn display_string<S>(&mut self, value: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        self.display(AdoData::String(value.as_ref().into()))
    }
}
