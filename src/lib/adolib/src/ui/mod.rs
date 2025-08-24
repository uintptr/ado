use crate::{data::types::AdoData, error::Result};

pub mod commands;
mod reddit;
pub mod status;

#[derive(Default)]
pub struct NopConsole {}

impl NopConsole {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConsoleDisplayTrait for NopConsole {
    fn start_spinner(&mut self) {}
    fn stop_spinner(&mut self) {}
    fn display<D>(&mut self, _data: D) -> Result<()>
    where
        D: AsRef<AdoData>,
    {
        Ok(())
    }
    fn display_string<S>(&mut self, _value: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        Ok(())
    }
}

pub trait ConsoleDisplayTrait {
    fn start_spinner(&mut self);
    fn stop_spinner(&mut self);
    fn display<D>(&mut self, data: D) -> Result<()>
    where
        D: AsRef<AdoData>;
    fn display_string<S>(&mut self, value: S) -> Result<()>
    where
        S: AsRef<str>;
}
