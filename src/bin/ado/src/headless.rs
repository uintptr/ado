use adolib::{console::ConsoleTrait, data::types::AdoData};
use anyhow::Result;

use crate::commands::UserCommands;

struct HeadlessConsole {}

impl ConsoleTrait for HeadlessConsole {
    fn error_message(&self, _message: &str) {}
    fn io(&self, _data: AdoData) -> Option<String> {
        None
    }
    fn enter_thinking(&self, _message: &str) {}
    fn leave_thinking(&self) {}
    fn print_markdown(&self, _s: &str) {}
}

pub fn headless_run(mut _commands: UserCommands) -> Result<()> {
    let _console = HeadlessConsole {};

    Ok(())
}
