use std::{
    fmt::Display,
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::mpsc::Sender,
};

pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

use adolib::{
    console::ConsoleTrait,
    data::types::{AdoData, AdoDataArtifact, AdoDataArtifactType, AdoDataStatus},
};
use anyhow::{Context, Result};
use log::{error, info};
use which::which;

use ansi_to_tui::IntoText;
use ratatui::{style::{Color, Style}, text::{Line, Span}};

use crate::tui_app::TuiEvent;

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
// TuiConsole — routes all output through a channel for the split-pane TUI
///////////////////////////////////////////////////////////////////////////////

pub struct TuiConsole {
    tx: Sender<TuiEvent>,
    glow: Option<PathBuf>,
}

impl TuiConsole {
    #[must_use]
    pub fn new(tx: Sender<TuiEvent>) -> Self {
        let glow = which("glow").ok();
        Self { tx, glow }
    }

    fn send(&self, event: TuiEvent) {
        let _ = self.tx.send(event);
    }

    fn send_lines(&self, text: &str) {
        for line in text.lines() {
            self.send(TuiEvent::PlainLine(line.to_string()));
        }
        if text.ends_with('\n') {
            self.send(TuiEvent::PlainLine(String::new()));
        }
    }

    /// Convert raw ANSI bytes from glow into owned ratatui `Line<'static>` objects.
    fn ansi_to_lines(bytes: &[u8]) -> Vec<Line<'static>> {
        match bytes.into_text() {
            Ok(text) => text
                .lines
                .into_iter()
                .map(|line| {
                    Line::from(
                        line.spans
                            .into_iter()
                            .map(|span| Span::styled(span.content.into_owned(), span.style))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect(),
            Err(_) => String::from_utf8_lossy(bytes)
                .lines()
                .map(|l| Line::from(l.to_string()))
                .collect(),
        }
    }

    /// Render `text` via glow (ANSI styled) if available, otherwise send as plain lines.
    /// stderr is suppressed so glow can never corrupt the TUI alternate screen.
    fn display_text(&self, text: &str) {
        if let Some(glow_path) = &self.glow {
            let result = Command::new(glow_path)
                .arg("-w")
                .arg("0")
                .arg("-s")
                .arg("dark")
                .env("CLICOLOR_FORCE", "1")
                .env("COLORTERM", "truecolor")
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
                let lines = Self::ansi_to_lines(&output.stdout);
                if !lines.is_empty() {
                    self.send(TuiEvent::StyledLines(lines));
                    return;
                }
            }
        }
        self.send_lines(text);
    }

    fn display_artifact(&self, artifact: &AdoDataArtifact) {
        match artifact.artifact_type {
            AdoDataArtifactType::Code => {
                let lang = artifact.language.as_deref().unwrap_or_default();
                self.display_text(&format!("```{}\n{}\n```", lang, artifact.content));
            }
            AdoDataArtifactType::Note => self.display_text(&artifact.content),
            _ => error!("artifact {} not handled in TuiConsole", artifact.artifact_type),
        }
    }

    fn process_partial_artifact(&self, artifact: &AdoDataArtifact) -> Option<String> {
        match artifact.artifact_type {
            AdoDataArtifactType::File => {
                if let Some(path) = &artifact.path {
                    self.send_lines(&format!(
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
                self.send_lines(&format!("executing \"{}\"", artifact.content));
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

impl ConsoleTrait for TuiConsole {
    fn error_message(&self, message: &str) {
        let line = Line::from(Span::styled(
            format!("Error: {message}"),
            Style::default().fg(Color::Red),
        ));
        self.send(TuiEvent::StyledLines(vec![line]));
    }

    fn io(&self, data: AdoData) -> Option<String> {
        let status_label = match data.meta.status {
            AdoDataStatus::Error => "Error",
            AdoDataStatus::Partial => "Partial",
            AdoDataStatus::Ok => "Ok",
        };
        self.send(TuiEvent::PlainLine(format!("{} {}", status_label, data.meta.intent)));

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
                self.send(TuiEvent::PlainLine(format!("Error: {}", data.response.message)));
                None
            }
            AdoDataStatus::Partial => {
                self.display_text(&data.response.message);
                let mut results = Vec::new();
                if let Some(artifacts) = &data.response.artifacts {
                    for artifact in artifacts {
                        if let Some(r) = self.process_partial_artifact(artifact) {
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

        self.send(TuiEvent::Separator);
        ret
    }

    fn print_markdown(&self, s: &str) {
        self.display_text(s);
        self.send(TuiEvent::Separator);
    }

    fn enter_thinking(&self, message: &str) {
        info!("TUI thinking: {message}");
        self.send(TuiEvent::ThinkingStart);
    }

    fn leave_thinking(&self) {
        self.send(TuiEvent::ThinkingStop);
    }
}
