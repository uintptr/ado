use std::{fs, io, path::PathBuf};

use adolib::console::ConsoleTrait;
use anyhow::Result;
use log::error;

use crate::{
    banner::render_banner,
    commands::UserCommands,
    input::{InputResult, read_line},
    terminal::{Console, PKG_NAME, PKG_VERSION},
};

pub fn load_history(history_file: &PathBuf) -> Vec<String> {
    fs::read_to_string(history_file)
        .unwrap_or_default()
        .lines()
        .map(String::from)
        .collect()
}

fn save_history(path: &PathBuf, history: &[String]) {
    let start = history.len().saturating_sub(1000);
    let content = history.get(start..).unwrap_or_default().join("\n");
    let _ = fs::write(path, content);
}

pub fn run(
    mut commands: UserCommands,
    mut history: Vec<String>,
    history_file: &PathBuf,
    command_names: &[String],
) -> Result<()> {
    if let Ok(banner) = render_banner(format!("{PKG_NAME} {PKG_VERSION}"), "pagga") {
        println!("{banner}");
    }

    let console = Console::new();

    while let InputResult::Line(line) = read_line("> ", &mut history, command_names)? {
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }
        history.push(trimmed.clone());

        // Echo the user input
        let mut stdout = io::stdout();
        let _ = crossterm::execute!(
            stdout,
            crossterm::style::SetForegroundColor(crossterm::style::Color::Green),
            crossterm::style::Print("> "),
            crossterm::style::ResetColor,
            crossterm::style::Print(format!("{trimmed}\n"))
        );

        if let Err(e) = commands.handler(&trimmed, &console) {
            error!("handler error: {e}");
            console.error_message(&format!("{e}"));
        }
        save_history(history_file, &history);
    }

    Ok(())
}
