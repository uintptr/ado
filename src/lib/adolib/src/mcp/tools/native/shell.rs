use std::{env, path::PathBuf};

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose};
use log::info;
use omcp::types::{BakedMcpToolTrait, McpParams};
use rustyline::hint::Hint;
use serde::Serialize;
use tokio::process::Command;

use crate::error::{Error, Result};

pub struct ToolShellExec {}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////
// GET IP
///////////////////////////////////////

#[derive(Serialize)]
struct TollShellResponse {
    exit_code: i32,
    b64_stdout: String,
    b64_stderr: String,
}

impl ToolShellExec {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl BakedMcpToolTrait for ToolShellExec {
    type Error = Error;

    async fn call(&mut self, params: &McpParams) -> Result<String> {
        let command_line = params.get_string("command_line")?;
        let wd = match params.get_string("working_directory") {
            Ok(v) => PathBuf::from(v),
            Err(_) => env::current_dir()?,
        };
        let timeout = params.get_int("timeout").unwrap_or(0);
        let shell = params.get_string("timeout").unwrap_or("/bin/bash");

        let args = shell_words::split(command_line)?;

        info!(
            "pwd={} shell={} timeout={} args={:?}",
            wd.display(),
            shell.display(),
            timeout,
            args
        );

        let out = Command::new(shell).arg("-c").args(args).current_dir(wd).output().await?;

        let exit_code = out.status.code().unwrap_or(-1);
        let b64_stdout = general_purpose::STANDARD.encode(out.stdout);
        let b64_stderr = general_purpose::STANDARD.encode(out.stderr);

        let result = TollShellResponse {
            exit_code,
            b64_stdout,
            b64_stderr,
        };
        Ok(serde_json::to_string(&result)?)
    }
}

#[cfg(test)]
mod tests {
    use log::info;
    use omcp::types::{BakedMcpToolTrait, McpParams};
    use serde_json::Value;

    use crate::{logging::logger::setup_logger, mcp::tools::ToolShellExec};

    #[tokio::test]
    async fn test_shell() {
        setup_logger(true).unwrap();

        let mut p = McpParams::new("shell_exec");
        p.add_argument("command_line", Value::String("id".to_string()));

        let mut sh = ToolShellExec::new();

        let res = sh.call(&p).await.unwrap();

        info!("{res}");
    }
}
