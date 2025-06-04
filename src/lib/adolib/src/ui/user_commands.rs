use std::io::{self, Write};

use crate::error::{Error, Result};

trait Command {
    fn name(&self) -> &'static str;
    fn desc(&self) -> &'static str;
    fn handler(&self) -> Result<String>;
}

struct CommandReset;
impl Command for CommandReset {
    fn name(&self) -> &'static str {
        "/reset"
    }
    fn desc(&self) -> &'static str {
        "Reset console and inputs"
    }

    fn handler(&self) -> Result<String> {
        print!("{esc}c", esc = 27 as char);
        io::stdout().flush()?;
        Err(Error::ResetInput)
    }
}

struct CommandQuit;
impl Command for CommandQuit {
    fn name(&self) -> &'static str {
        "/quit"
    }
    fn desc(&self) -> &'static str {
        "Deuces ☮"
    }

    fn handler(&self) -> Result<String> {
        Err(Error::EOF)
    }
}

#[derive(Default)]
pub struct UserCommands {
    handlers: Vec<Box<dyn Command>>,
}

impl UserCommands {
    pub fn new() -> Self {
        let handlers: Vec<Box<dyn Command>> = vec![Box::new(CommandReset), Box::new(CommandQuit)];

        Self { handlers }
    }

    pub fn handler(&self, line: &str) -> Result<String> {
        for c in self.handlers.iter() {
            if c.name() == line {
                return c.handler();
            }
        }

        Err(Error::CommandNotFound {
            command: line.to_string(),
        })
    }

    pub fn list_commands(&self) -> Vec<(&'static str, &'static str)> {
        let mut command_names = Vec::new();

        for c in self.handlers.iter() {
            command_names.push((c.name(), c.desc()));
        }

        command_names
    }
}
