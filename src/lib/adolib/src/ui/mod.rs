use crate::{data::types::AdoData, error::Result};

pub mod commands;
mod reddit;
pub mod status;

pub struct NopConsole {}

impl ConsoleDisplayTrait for NopConsole {
    fn start_spinner(&mut self) {}
    fn stop_spinner(&mut self) {}
    fn display(&self, _data: AdoData) -> Result<()> {
        Ok(())
    }
    fn display_string<S>(&self, _value: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        Ok(())
    }
}

pub trait ConsoleDisplayTrait {
    fn start_spinner(&mut self);
    fn stop_spinner(&mut self);
    fn display(&self, data: AdoData) -> Result<()>;
    fn display_string<S>(&self, value: S) -> Result<()>
    where
        S: AsRef<str>;
}
