#![allow(unused)] // while working on the wasm build

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

pub struct UserCommands {
    commands: Vec<Box<dyn Command>>,
}

impl UserCommands {
    pub fn new() -> Self {
        let commands: Vec<Box<dyn Command>> = vec![Box::new(CommandReset), Box::new(CommandQuit)];

        Self { commands }
    }

    fn display_help(&self) -> String {
        let mut help = Vec::new();

        help.push("# Help".to_string());

        help.push(format!("* `{:<9}` This help", "/help"));

        for c in self.commands.iter() {
            help.push(format!("* `{:<9}` {}", c.name(), c.desc()));
        }

        help.join("\n")
    }

    pub fn handler(&self, line: &str) -> Result<String> {
        if line.eq("/help") {
            return Ok(self.display_help());
        }

        for c in self.commands.iter() {
            if c.name() == line {
                return c.handler();
            }
        }

        Err(Error::CommandNotFound {
            command: line.to_string(),
        })
    }

    pub fn list_commands(&self) -> Vec<&'static str> {
        let mut command_names = Vec::new();

        command_names.push("/help");

        for c in self.commands.iter() {
            command_names.push(c.name());
        }

        command_names
    }
}
