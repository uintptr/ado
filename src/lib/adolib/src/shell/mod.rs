use std::process::{Command, Stdio};

use log::info;

use crate::{
    data::{AdoData, ShellExit},
    error::{Error, Result},
};

#[derive(Default)]
pub struct AdoShell {}

impl AdoShell {
    pub fn new() -> Self {
        Self {}
    }

    pub fn exec(&self, command_line: &str) -> Result<AdoData> {
        let comp = shell_words::split(command_line)?;

        info!("executing: {command_line}");

        let program = comp.first().ok_or(Error::CommandNotFound {
            command: command_line.to_string(),
        })?;

        let child = Command::new(program)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&comp[1..])
            .spawn()?;

        let exit = ShellExit::from_child(child)?;

        Ok(AdoData::Shell(exit))
    }
}

#[cfg(test)]
mod tests {

    use log::info;

    use crate::logging::logger::setup_logger;

    use super::AdoShell;

    #[test]
    fn test_shell() {
        setup_logger(true).unwrap();
        let sh = AdoShell::new();
        let data = sh.exec("uname -a").unwrap();
        info!("{data:?}");
    }
}
