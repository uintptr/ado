use std::io::{self, Write};

use crate::error::{Error, Result};

pub struct UserCommands {}

impl UserCommands {
    pub fn new() -> Self {
        Self {}
    }

    fn help(&self) -> Result<String> {
        let help = r#"# Commands:
        * help
        * reset
        * history
        * quit
        "#;

        Ok(help.to_string())
    }

    fn reset(&self) -> Result<String> {
        print!("{esc}c", esc = 27 as char);
        io::stdout().flush()?;
        Err(Error::ResetInput)
    }

    pub fn process(&self, line: &str) -> Result<String> {
        match line {
            "help" => self.help(),
            "reset" => self.reset(),
            "quit" => Err(Error::EOF),
            "history" => Err(Error::NotImplemented),
            _ => Err(Error::CommandNotFound),
        }
    }
}
