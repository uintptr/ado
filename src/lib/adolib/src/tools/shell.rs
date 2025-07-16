use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Serialize, Serializer};
use std::process::{Command, Stdio};

use log::info;

use crate::{
    data::{AdoData, ShellExit},
    error::{Error, Result},
};

use super::function_args::FunctionArgs;

#[derive(Serialize)]
pub struct ShellOutput {
    exit_code: i32,
    #[serde(serialize_with = "base64_serializer")]
    b64_stdout: Vec<u8>,
    #[serde(serialize_with = "base64_serializer")]
    b64_stderr: Vec<u8>,
}

pub fn base64_serializer<S>(bytes: &Vec<u8>, serializer: S) -> core::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&BASE64_STANDARD.encode(bytes))
}

pub struct FunctionsShell {}

impl FunctionsShell {
    pub fn new() -> Self {
        Self {}
    }

    pub fn shell(&self, command_line: &str) -> Result<AdoData> {
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

    pub fn shell_exec(&self, args: &FunctionArgs) -> Result<AdoData> {
        let line = args.get_string("command_line")?;
        self.shell(line)
    }
}

#[cfg(test)]
mod tests {

    use crate::logging::logger::setup_logger;

    use super::FunctionsShell;

    #[test]
    fn test_shell() {
        setup_logger(true).unwrap();

        let shell = FunctionsShell::new();

        shell.shell("ls -l /").unwrap();
    }
}
