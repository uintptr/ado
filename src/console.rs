use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use log::{info, warn};
use which::which;

use crate::error::Result;

pub struct ConsoleUI {
    glow: Option<PathBuf>,
}

impl ConsoleUI {
    pub fn new() -> Self {
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

        Self { glow }
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

    pub fn display_text(&self, text: &str) -> Result<()> {
        match &self.glow {
            Some(v) => self.display_glow(&v, text),
            None => self.display_boring(text),
        }
    }
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
