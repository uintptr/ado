use std::io::{self, BufRead};

use adolib::{console::ConsoleTrait, data::types::AdoData};
use anyhow::Result;
use log::error;

use crate::commands::UserCommands;

struct HeadlessConsole {}

impl ConsoleTrait for HeadlessConsole {
    fn error_message(&self, message: &str) {
        eprintln!("{message}");
    }

    fn io(&self, data: AdoData) -> Option<String> {
        println!("{data}");
        None
    }

    fn enter_thinking(&self, _message: &str) {}
    fn leave_thinking(&self) {}

    fn print_markdown(&self, s: &str) {
        println!("{s}");
    }
}

pub fn headless_run(mut commands: UserCommands) -> Result<()> {
    let console = HeadlessConsole {};
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }

        if let Err(e) = commands.handler(&trimmed, &console) {
            error!("handler error: {e}");
            console.error_message(&format!("{e}"));
        }
    }

    Ok(())
}
