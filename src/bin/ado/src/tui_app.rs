use std::{io, path::Path};

use adolib::console::ConsoleTrait;
use anyhow::Result;
use log::error;

use crate::{
    banner::render_banner,
    commands::UserCommands,
    input::{self, InputResult},
    terminal::{Console, PKG_NAME, PKG_VERSION},
};

pub fn run(
    mut commands: UserCommands,
    history_file: &Path,
    command_names: Vec<String>,
) -> Result<()> {
    if let Ok(banner) = render_banner(format!("{PKG_NAME} {PKG_VERSION}"), "pagga") {
        println!("{banner}");
    }

    let console = Console::new();
    let mut editor = input::create_editor(history_file, command_names)?;

    while let InputResult::Line(line) = input::read_line(&mut editor)? {
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }

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
    }

    Ok(())
}
