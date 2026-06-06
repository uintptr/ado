use std::io::{self, BufRead, Write};

use adolib::{console::ConsoleTrait, data::types::AdoData};
use anyhow::Result;
use log::error;
use serde::Serialize;

use crate::commands::UserCommands;

/// Newline-delimited JSON protocol spoken over stdout in headless mode.
///
/// Exactly one compact (single-line) JSON object is written per message, so the
/// webapp can split stdout on `\n` and parse each line independently. Every
/// message carries a `type` discriminator.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum HeadlessMessage<'a> {
    /// A structured LLM response.
    Data { data: &'a AdoData },
    /// Plain markdown emitted by built-in commands (e.g. `/help`, `/models`).
    Markdown { text: &'a str },
    /// An error message.
    Error { message: &'a str },
}

impl HeadlessMessage<'_> {
    /// Serialize as one line of JSON and flush so the client sees it promptly.
    fn emit(&self) {
        let line = match serde_json::to_string(self) {
            Ok(line) => line,
            Err(e) => {
                error!("failed to serialize headless message: {e}");
                return;
            }
        };

        let mut stdout = io::stdout().lock();
        if let Err(e) = writeln!(stdout, "{line}").and_then(|()| stdout.flush()) {
            error!("failed to write headless message: {e}");
        }
    }
}

struct HeadlessConsole {}

impl ConsoleTrait for HeadlessConsole {
    fn error_message(&self, message: &str) {
        HeadlessMessage::Error { message }.emit();
    }

    fn io(&self, data: AdoData) -> Option<String> {
        HeadlessMessage::Data { data: &data }.emit();
        None
    }

    fn enter_thinking(&self, _message: &str) {}
    fn leave_thinking(&self) {}

    fn print_markdown(&self, s: &str) {
        HeadlessMessage::Markdown { text: s }.emit();
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
