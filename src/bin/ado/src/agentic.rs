//! Execution of agentic (`partial`) artifacts — running shell commands and
//! writing files on the host. Shared by the TUI and headless consoles so both
//! drive the same agentic loop. Each caller passes a `notify` callback to
//! surface the action in its own way (coloured line in the TUI, structured
//! message in headless).

use std::{fmt::Display, fs, process::Command};

use adolib::data::types::{AdoDataArtifact, AdoDataArtifactType};
use anyhow::{Context, Result};
use log::error;

/// Run a shell command line and return its combined stdout + stderr.
pub fn handler_command<S>(cmd_line: S) -> Result<String>
where
    S: AsRef<str> + Display,
{
    let args = shell_words::split(cmd_line.as_ref()).with_context(|| format!("Unable to split {cmd_line}"))?;

    let cmd = args.first().with_context(|| format!("Empty command: {cmd_line}"))?;
    let out = Command::new(cmd)
        .args(args.get(1..).unwrap_or_default())
        .output()
        .with_context(|| format!("Unable to execute {cmd_line}"))?;

    let mut data = String::from_utf8_lossy(&out.stdout).into_owned();
    if !out.stderr.is_empty() {
        data.push_str(&String::from_utf8_lossy(&out.stderr));
    }

    Ok(data)
}

/// Execute a single `command` or `file` artifact from a `partial` response.
///
/// `notify` is called with a human-readable description of the action before it
/// runs. Returns the result to feed back to the model so the loop can continue,
/// or `None` for artifact types that aren't executable.
pub fn execute_partial_artifact(artifact: &AdoDataArtifact, notify: &dyn Fn(&str)) -> Option<String> {
    match &artifact.artifact_type {
        AdoDataArtifactType::File => {
            let Some(path) = &artifact.path else {
                return Some("File path is missing".into());
            };
            notify(&format!("writing {} bytes to {}", artifact.content.len(), path.display()));
            match fs::write(path, artifact.content.as_bytes()) {
                Ok(()) => Some(format!("{} was successfully written to disk", path.display())),
                Err(e) => Some(format!("Unable to write {} to disk. Error: {e}", path.display())),
            }
        }
        AdoDataArtifactType::Command => {
            notify(&format!("executing \"{}\"", artifact.content));
            match handler_command(&artifact.content) {
                Ok(v) => Some(v),
                Err(e) => Some(format!("Unable to execute {}. Error: {e}", artifact.content)),
            }
        }
        other => {
            error!("partial artifact type not executable: {other}");
            None
        }
    }
}
