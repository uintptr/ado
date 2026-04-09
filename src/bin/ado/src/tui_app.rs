use std::path::Path;

use adolib::console::ConsoleTrait;
use anyhow::Result;
use log::error;

use crate::{
    banner::render_banner,
    commands::UserCommands,
    input::{self, InputResult},
    terminal::{Console, PKG_NAME, PKG_VERSION},
};

pub fn run(mut commands: UserCommands, history_file: &Path) -> Result<()> {
    // Build the list of command names and history path before moving commands
    let command_names: Vec<String> = commands.list_commands().iter().map(|c| c.name().to_string()).collect();

    if let Ok(banner) = render_banner(format!("{} {PKG_VERSION}", PKG_NAME.to_uppercase())) {
        println!("{banner}");
    }

    let console = Console::new();
    let mut editor = input::create_editor(history_file, command_names)?;

    loop {
        let model = commands.current_model();
        match input::read_line(&mut editor, &model)? {
            InputResult::Line(line) => {
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }
                if let Err(e) = commands.handler(&trimmed, &console) {
                    error!("handler error: {e}");
                    console.error_message(&format!("{e}"));
                }
            }
            InputResult::Eof => break,
        }
    }

    Ok(())
}
