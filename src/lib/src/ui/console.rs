use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use log::{error, info, warn};
use rustyline::{DefaultEditor, error::ReadlineError};
#[cfg(not(target_arch = "wasm32"))]
use which::which;

use crate::error::{Error, Result};

use super::UiTrait;

pub struct ConsoleUI {
    glow: Option<PathBuf>,
    rl: DefaultEditor,
}

impl ConsoleUI {
    pub fn new() -> Result<Self> {
        let glow = match which("glow") {
            Ok(v) => {
                info!("glow is @ {}", v.display());
                Some(v)
            }
            Err(_) => {
                warn!("glow is not installed");
                None
            }
        };

        let rl = DefaultEditor::new()?;

        Ok(Self { glow, rl })
    }

    fn display_glow(&self, glow: &Path, text: &str) -> Result<()> {
        let mut child = Command::new(glow).stdin(Stdio::piped()).spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(text.as_bytes())?;
        }

        child.wait()?;

        Ok(())
    }

    fn display_boring(&self, text: &str) -> Result<()> {
        print!("{text}");
        Ok(())
    }
}

impl UiTrait for ConsoleUI {
    fn display(&self, text: &str) -> Result<()> {
        match &self.glow {
            Some(v) => self.display_glow(v, text),
            None => self.display_boring(text),
        }
    }

    fn readline(&mut self) -> Result<String> {
        loop {
            let res = match self.rl.readline("> ") {
                Ok(line) => Ok(line),
                // CTRL+D
                Err(ReadlineError::Eof) => Err(Error::EOF),
                // CTRL+C
                Err(ReadlineError::Interrupted) => Err(Error::EOF),
                // ¯\_(ツ)_/¯
                Err(e) => {
                    error!("{e}");
                    Err(e.into())
                }
            };

            if let Ok(line) = &res {
                if line.is_empty() {
                    continue;
                }
                break res;
            } else {
                break res;
            }
        }
    }

    /*
    fn readline(&self) -> Result<String> {
        let readline = self.readline(">>")?;

        let mut query = String::new();

        loop {
            print!("> ");
            io::stdout().flush()?;
            //
            // use readline or something so we can use CTRL+ENTER to return
            //
            io::stdin().read_line(&mut query)?;

            let query = query.trim_end_matches('\n');

            if query.is_empty() {
                warn!("empty input...");
                continue;
            }
            break;
        }

        Ok(query.trim_end_matches('\n').to_string())
    }
    */
}

#[cfg(test)]
mod tests {
    use crate::staples::setup_logger;

    use super::ConsoleUI;

    #[test]
    fn display_text() {
        setup_logger(true).unwrap();

        let console = ConsoleUI::new();

        console.display_text("hello world").unwrap();
    }
}
