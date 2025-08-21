use crate::{
    data::types::{AdoData, AdoDataMarkdown},
    error::Result,
    ui::ConsoleDisplayTrait,
    wasm::entry::{WasmQueryResponse, wasm_display},
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
    fn start_spinner(&mut self) {}
    fn stop_spinner(&mut self) {}
    fn display(&mut self, data: AdoData) -> Result<()> {
        self.data_list.push(data);
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
    fn start_spinner(&mut self) {}
    fn stop_spinner(&mut self) {}
    fn display(&mut self, data: AdoData) -> Result<()> {
        let resp = WasmQueryResponse {
            data: data.clone(),
            markdown: data.to_markdown()?,
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
