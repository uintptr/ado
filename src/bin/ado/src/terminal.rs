use std::{
    fmt::Display,
    fs,
    io::{self, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

use adolib::{
    console::ConsoleTrait,
    data::types::{AdoData, AdoDataArtifact, AdoDataArtifactType, AdoDataStatus},
};
use anyhow::{Context, Result};
use crossterm::{execute, style, terminal};
use log::{error, info};
use which::which;

use crate::spinner::AdoSpinner;

fn handler_command<S>(cmd_line: S) -> Result<String>
where
    S: AsRef<str> + Display,
{
    let args = shell_words::split(cmd_line.as_ref()).with_context(|| format!("Unable to split {cmd_line}"))?;

    let cmd = args.first().with_context(|| format!("Empty command: {cmd_line}"))?;
    let out = Command::new(cmd)
        .args(args.get(1..).unwrap_or_default())
        .output()
        .with_context(|| format!("Unable to execute {cmd_line}"))?;

    let data = String::from_utf8(out.stdout).context("Unable to convert stdout to a string")?;

    Ok(data)
}

///////////////////////////////////////////////////////////////////////////////
// Console — prints directly to stdout
///////////////////////////////////////////////////////////////////////////////

pub struct Console {
    glow: Option<PathBuf>,
    spinner: AdoSpinner,
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

fn print_separator() {
    let width = terminal::size().map(|(w, _)| w as usize).unwrap_or(80);
    let sep = "─".repeat(width.min(500));
    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        style::SetForegroundColor(style::Color::DarkGrey),
        style::Print(format!("{sep}\n")),
        style::ResetColor
    );
}

fn print_lines<S>(text: S)
where
    S: AsRef<str>,
{
    let mut stdout = io::stdout();
    let _ = stdout.write_all(text.as_ref().as_bytes());
    if !text.as_ref().ends_with('\n') {
        let _ = stdout.write_all(b"\n");
    }
    let _ = stdout.flush();
}

impl Console {
    #[must_use]
    pub fn new() -> Self {
        let glow = which("glow").ok();
        Self {
            glow,
            spinner: AdoSpinner::new(),
        }
    }

    /// Render `text` via glow if available, otherwise print plain.
    fn display_text(&self, text: &str) {
        if let Some(glow_path) = &self.glow {
            let result = Command::new(glow_path)
                .arg("-w")
                .arg("0")
                .arg("-s")
                .arg("dark")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .and_then(|mut child| {
                    if let Some(stdin) = child.stdin.as_mut() {
                        let _ = stdin.write_all(text.as_bytes());
                    }
                    child.wait_with_output()
                });

            if let Ok(output) = result
                && output.status.success()
                && !output.stdout.is_empty()
            {
                let mut stdout = io::stdout();
                let _ = stdout.write_all(&output.stdout);
                let _ = stdout.flush();
                return;
            }
        }
        print_lines(text);
    }

    fn display_artifact(&self, artifact: &AdoDataArtifact) {
        match artifact.artifact_type {
            AdoDataArtifactType::Code => {
                let lang = artifact.language.as_deref().unwrap_or_default();
                self.display_text(&format!("```{}\n{}\n```", lang, artifact.content));
            }
            AdoDataArtifactType::Note => self.display_text(&artifact.content),
            _ => error!("artifact {} not handled in Console", artifact.artifact_type),
        }
    }

    fn process_partial_artifact(artifact: &AdoDataArtifact) -> Option<String> {
        match artifact.artifact_type {
            AdoDataArtifactType::File => {
                if let Some(path) = &artifact.path {
                    print_lines(format!(
                        "Writing {} bytes to {}",
                        artifact.content.len(),
                        path.display()
                    ));
                    match fs::write(path, artifact.content.as_bytes()) {
                        Ok(()) => Some(format!("{} was successfully written to disk", path.display())),
                        Err(e) => Some(format!("Unable to write {} to disk. Error: {e}", path.display())),
                    }
                } else {
                    Some("File path is missing".into())
                }
            }
            AdoDataArtifactType::Command => {
                print_lines(format!("executing \"{}\"", artifact.content));
                match handler_command(&artifact.content) {
                    Ok(v) => Some(v),
                    Err(e) => Some(format!("Unable to execute {}. Error: {e}", artifact.content)),
                }
            }
            _ => {
                error!("unhandled type: {}", artifact.artifact_type);
                None
            }
        }
    }
}

impl ConsoleTrait for Console {
    fn error_message(&self, message: &str) {
        let mut stdout = io::stdout();
        let _ = execute!(
            stdout,
            style::SetForegroundColor(style::Color::Red),
            style::Print(format!("Error: {message}\n")),
            style::ResetColor
        );
    }

    fn io(&self, data: AdoData) -> Option<String> {
        self.spinner.stop();

        let status_label = match data.meta.status {
            AdoDataStatus::Error => "Error",
            AdoDataStatus::Partial => "Partial",
            AdoDataStatus::Ok => "Ok",
        };

        println!("{status_label} {}", data.meta.intent);

        let ret = match data.meta.status {
            AdoDataStatus::Ok => {
                self.display_text(&data.response.message);
                if let Some(artifacts) = &data.response.artifacts {
                    for artifact in artifacts {
                        self.display_artifact(artifact);
                    }
                }
                None
            }
            AdoDataStatus::Error => {
                self.error_message(&data.response.message);
                None
            }
            AdoDataStatus::Partial => {
                self.display_text(&data.response.message);
                let mut results = Vec::new();
                if let Some(artifacts) = &data.response.artifacts {
                    for artifact in artifacts {
                        if let Some(r) = Console::process_partial_artifact(artifact) {
                            results.push(r);
                        }
                    }
                }
                if results.is_empty() {
                    None
                } else {
                    Some(results.join(" "))
                }
            }
        };

        print_separator();
        ret
    }

    fn print_markdown(&self, s: &str) {
        self.spinner.stop();
        self.display_text(s);
        print_separator();
    }

    fn enter_thinking(&self, message: &str) {
        info!("Thinking: {message}");
        self.spinner.start();
    }

    fn leave_thinking(&self) {
        self.spinner.stop();
    }
}
