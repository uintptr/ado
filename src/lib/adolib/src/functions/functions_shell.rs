use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Serialize, Serializer};
use std::{
    io::Read,
    process::{Command, Stdio},
};

use log::{error, info};

use crate::error::{Error, Result};

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

    pub fn shell(&self, command_line: &str) -> Result<String> {
        let comp = shell_words::split(command_line)?;

        info!("executing: {}", command_line);

        let program = comp.first().ok_or(Error::CommandNotFound {
            command: command_line.to_string(),
        })?;

        let mut child = Command::new(program)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&comp[1..])
            .spawn()?;

        let exit = child.wait()?;

        let stdout = match child.stdout.as_mut() {
            Some(v) => {
                let mut buf = Vec::new();
                v.read_to_end(&mut buf)?;
                buf
            }
            None => vec![],
        };

        let stderr = match child.stderr.as_mut() {
            Some(v) => {
                let mut buf = Vec::new();
                v.read_to_end(&mut buf)?;
                buf
            }
            None => vec![],
        };

        let exit_code = exit.code().unwrap_or(1);

        match exit_code {
            0 => info!("{} returned 0", program),
            _ => error!("{} returned {}", program, exit_code),
        }

        let output = ShellOutput {
            exit_code,
            b64_stdout: stdout,
            b64_stderr: stderr,
        };

        let output_json = serde_json::to_string(&output)?;

        Ok(output_json)
    }

    pub fn shell_exec(&self, args: &FunctionArgs) -> Result<String> {
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
