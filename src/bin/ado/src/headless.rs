use std::io::{self, BufRead, Write};

use adolib::{
    console::ConsoleTrait,
    data::types::{AdoData, AdoDataStatus},
};
use anyhow::Result;
use log::error;
use serde::Serialize;

use crate::{agentic, commands::UserCommands};

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
    /// A progress note for an agentic action (running a command, writing a file).
    Action { text: &'a str },
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

        // A `partial` response carries artifacts (commands/files) for us to
        // execute inside the container; run them and feed the results back so
        // the agentic loop continues.
        if matches!(data.meta.status, AdoDataStatus::Partial) {
            let mut results = Vec::new();
            if let Some(artifacts) = &data.response.artifacts {
                for artifact in artifacts {
                    let notify = |text: &str| HeadlessMessage::Action { text }.emit();
                    if let Some(r) = agentic::execute_partial_artifact(artifact, &notify) {
                        results.push(r);
                    }
                }
            }
            if !results.is_empty() {
                return Some(results.join(" "));
            }
        }

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
