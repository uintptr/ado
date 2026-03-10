use std::{
    fmt::Display,
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

use adolib::{
    console::ConsoleTrait,
    data::types::{AdoData, AdoDataArtifact, AdoDataArtifactType, AdoDataMeta, AdoDataStatus},
    error::Error,
};
use anyhow::{Context, Result};
use colored::Colorize;
use log::{error, info, warn};
use which::which;

use crate::{banner::display_banner, commands::UserCommands, input, spinner::AdoSpinner};

pub struct TerminalConsole {
    glow: Option<PathBuf>,
    history: Vec<String>,
    history_file: PathBuf,
    commands: Vec<String>,
    spinner: AdoSpinner,
}

///////////////////////////////////////////////////////////////////////////////
// FUNC
///////////////////////////////////////////////////////////////////////////////

pub fn separator() {
    let width = crossterm::terminal::size().map(|(w, _)| w as usize).unwrap_or(80);
    println!("{}", "─".repeat(width).green());
}

fn load_history(path: &PathBuf) -> Vec<String> {
    fs::read_to_string(path).unwrap_or_default().lines().map(String::from).collect()
}

fn save_history(path: &PathBuf, history: &[String]) {
    let start = history.len().saturating_sub(1000);
    let content = history.get(start..).unwrap_or_default().join("\n");
    let _ = fs::write(path, content);
}

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
// IMPL
///////////////////////////////////////////////////////////////////////////////
impl TerminalConsole {
    pub fn new(commands: &UserCommands) -> Result<Self> {
        let glow = if let Ok(v) = which("glow") {
            info!("glow is @ {}", v.display());
            Some(v)
        } else {
            warn!("glow is not installed");
            None
        };

        let config_dir = dirs::config_dir().ok_or(Error::ConfigNotFound)?;
        let history_file = config_dir.join("history.txt");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        let history = load_history(&history_file);

        let mut command_names = Vec::new();
        for c in commands.list_commands() {
            command_names.push(c.name().to_string());
        }

        // Show banner (no full screen clear, preserves scrollback)
        let _ = display_banner(format!("{PKG_NAME} {PKG_VERSION}"), "pagga");

        let spinner = AdoSpinner::new();

        Ok(Self {
            glow,
            history,
            history_file,
            commands: command_names,
            spinner,
        })
    }

    fn display_glow<S>(&self, text: S) -> Result<()>
    where
        S: AsRef<str> + Display,
    {
        if let Some(glob_path) = &self.glow {
            let mut child = Command::new(glob_path)
                .arg("-w")
                .arg("0")
                .arg("-s")
                .arg("dark")
                .stdin(Stdio::piped())
                .spawn()?;

            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(text.as_ref().as_bytes())?;
            }

            child.wait()?;

            Ok(())
        } else {
            Err(Error::NotFound.into())
        }
    }

    pub fn display_string<S>(&self, text: S) -> Result<()>
    where
        S: AsRef<str> + Display,
    {
        if self.glow.is_some() {
            self.display_glow(text)
        } else {
            print!("{text}");
            Ok(())
        }
    }

    pub fn read_input(&mut self) -> Result<String> {
        loop {
            match input::read_line("> ", &mut self.history, &self.commands)? {
                input::InputResult::Line(line) => {
                    let line = line.trim().to_string();
                    if line.is_empty() {
                        continue;
                    }
                    self.history.push(line.clone());
                    return Ok(line);
                }
                input::InputResult::Eof => {
                    return Err(Error::EOF.into());
                }
            }
        }
    }

    fn display_meta(meta: &AdoDataMeta) {
        let status_str = match meta.status {
            AdoDataStatus::Error => "Error".red(),
            AdoDataStatus::Partial => "Partial".green(),
            AdoDataStatus::Ok => "Ok".green(),
        };

        let intent = meta.intent.clone();

        println!("{} {}", status_str, intent.yellow());
    }

    fn display_data_code(&self, artifact: &AdoDataArtifact) -> Result<()> {
        let lang = artifact.language.as_deref().unwrap_or_default();

        let data = format!("```{}\n{}\n```", lang, artifact.content);

        self.display_string(data)
    }

    #[allow(clippy::todo)]
    fn display_data_artifact(&self, artifact: &AdoDataArtifact) -> Result<()> {
        match artifact.artifact_type {
            AdoDataArtifactType::Code => self.display_data_code(artifact),
            AdoDataArtifactType::Note => self.display_string(&artifact.content),
            _ => {
                error!("artifact {} was not supported", artifact.artifact_type);
                todo!()
            }
        }
    }

    fn display_data_response(&self, data: AdoData) -> Option<String> {
        println!("{}", data.response.message);

        if let Some(artifacts) = data.response.artifacts {
            for arti in artifacts {
                if let Err(e) = self.display_data_artifact(&arti) {
                    error!("error displaying data. ({e})");
                }
            }
        }

        None
    }

    fn display_data_error(data: &AdoData) -> Option<String> {
        let err_str = format!("Error: {}", data.response.message);
        println!("{}", err_str.red());
        None
    }

    fn process_partial_artifact(&self, artifact: &AdoDataArtifact) -> Option<String> {
        match artifact.artifact_type {
            AdoDataArtifactType::File => {
                if let Some(path) = &artifact.path {
                    match fs::write(path, artifact.content.as_bytes()) {
                        Ok(()) => Some(format!("{} was successfully written to disk", path.display())),
                        Err(e) => Some(format!("Unable to write {} to disk. Error: {e}", path.display())),
                    }
                } else {
                    Some("File path is missing".into())
                }
            }
            AdoDataArtifactType::Command => match handler_command(&artifact.content) {
                Ok(v) => Some(v),
                Err(e) => Some(format!("Unable to execute {}. Error: {e}", artifact.content)),
            },
            _ => {
                error!("unhandled type: {}", artifact.artifact_type);
                None
            }
        }
    }

    fn process_data_partial(&self, data: AdoData) -> Option<String> {
        if let Err(e) = self.display_string(data.response.message) {
            error!("display failure. ({e})");
        }

        let mut response_entries = vec![];

        if let Some(artifact) = &data.response.artifacts {
            for arti in artifact {
                if let Some(response) = self.process_partial_artifact(arti) {
                    response_entries.push(response);
                }
            }
        }

        if response_entries.is_empty() {
            None
        } else {
            Some(response_entries.join(" "))
        }
    }
}

impl Drop for TerminalConsole {
    fn drop(&mut self) {
        save_history(&self.history_file, &self.history);
    }
}

impl ConsoleTrait for TerminalConsole {
    fn io(&self, data: AdoData) -> Option<String> {
        TerminalConsole::display_meta(&data.meta);

        let ret = match data.meta.status {
            AdoDataStatus::Ok => self.display_data_response(data),
            AdoDataStatus::Error => TerminalConsole::display_data_error(&data),
            AdoDataStatus::Partial => self.process_data_partial(data),
        };
        separator();
        ret
    }

    fn print_markdown(&self, s: &str) {
        if let Err(e) = self.display_string(s) {
            error!("{e}");
        }
        separator();
    }

    fn enter_thinking(&self, message: &str) {
        info!("spinning {message}");
        self.spinner.start();
    }
    fn leave_thinking(&self) {
        self.spinner.stop();
    }
}
