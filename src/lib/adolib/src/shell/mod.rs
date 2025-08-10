use std::process::{Command, Stdio};

use log::info;

use crate::{
    data::types::AdoData,
    error::{Error, Result},
};

use std::{io::Read, process::Child, time::Instant};

use serde::{Deserialize, Serialize};

use crate::data::types::AdoDataMarkdown;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShellExit {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
    pub execution_time: u64,
}

impl AdoDataMarkdown for ShellExit {
    fn to_markdown(self) -> Result<String> {
        let mut lines = Vec::new();

        lines.push("# Shell Execution".to_string());

        lines.push(format!(" * status: {}", self.exit_code));
        lines.push(format!(" * time: {}", self.execution_time));

        if !self.stdout.is_empty() {
            lines.push("## stdout".to_string());
            lines.push(format!("```\n{}\n```", self.stdout));
        }

        if !self.stderr.is_empty() {
            lines.push("## stderr".to_string());
            lines.push(format!("```\n{}\n```", self.stderr));
        }

        Ok(lines.join("\n"))
    }
}

impl ShellExit {
    pub fn from_child(mut child: Child) -> Result<ShellExit> {
        let start = Instant::now();

        let exit = child.wait()?;

        let duration = start.elapsed();

        let stdout = match child.stdout.as_mut() {
            Some(v) => {
                let mut buf = Vec::new();
                v.read_to_end(&mut buf)?;
                buf
            }
            None => vec![],
        };

        let stdout = String::from_utf8(stdout).unwrap_or("invalid stdout data".to_string());

        let stderr = match child.stderr.as_mut() {
            Some(v) => {
                let mut buf = Vec::new();
                v.read_to_end(&mut buf)?;
                buf
            }
            None => vec![],
        };

        let stderr = String::from_utf8(stderr).unwrap_or("invalid stderr data".to_string());

        let exit_code = exit.code().unwrap_or(1);

        Ok(ShellExit {
            stdout,
            stderr,
            exit_code,
            timed_out: false,
            execution_time: duration.as_secs(),
        })
    }
}

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
