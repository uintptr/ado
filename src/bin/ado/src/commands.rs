use std::fmt::Display;

use adolib::error::Result;
use log::info;

use crate::console::TerminalConsole;

pub struct UserCommands {}

pub struct UserCommandEntry {
    pub name: String,
    pub aliases: Vec<String>,
}

impl UserCommands {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn handler<S>(&self, input: S, mut _console: &TerminalConsole) -> Result<()>
    where
        S: AsRef<str> + Display,
    {
        info!("input: {input}");
        Ok(())
    }

    pub fn list_commands(&self) -> Vec<UserCommandEntry> {
        vec![]
    }
}
