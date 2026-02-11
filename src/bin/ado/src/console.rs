use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Stdio,
};

pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

use adolib::{
    const_vars::{DIRS_APP, DIRS_ORG, DIRS_QUALIFIER},
    data::types::{AdoData, AdoDataMarkdown},
    error::{Error, Result},
    ui::{ConsoleDisplayTrait, commands::UserCommands},
};
use colored;
use colored::Colorize;
use directories::ProjectDirs;
use log::{error, info, warn};
use spinner::{SpinnerBuilder, SpinnerHandle};
use which::which;

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{Cmd, CompletionType, Config, Editor, KeyCode, KeyEvent, Modifiers, Movement};
use rustyline::{Context, Helper, Hinter, Validator};
use rustyline::{Highlighter, hint::HistoryHinter};

use crate::banner::display_banner;

struct CommandCompleter {
    commands: Vec<String>,
    file_completer: FilenameCompleter,
}

impl CommandCompleter {
    fn new(commands: Vec<String>) -> Self {
        Self {
            commands,
            file_completer: FilenameCompleter::new(),
        }
    }
}

impl Completer for CommandCompleter {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<Pair>)> {
        // If we're still on the first word (no space before cursor), complete commands
        if !line[..pos].contains(' ') {
            let prefix = &line[..pos];
            let matches: Vec<Pair> = self
                .commands
                .iter()
                .filter(|cmd| cmd.starts_with(prefix))
                .map(|cmd| Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                })
                .collect();

            if !matches.is_empty() {
                return Ok((0, matches));
            }
        }

        // Fall back to filename completion
        self.file_completer.complete(line, pos, ctx)
    }
}

#[derive(Helper, Highlighter, Hinter, Validator)]
struct MyHelper {
    completer: CommandCompleter,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<Pair>)> {
        self.completer.complete(line, pos, ctx)
    }
}

pub struct TerminalConsole {
    glow: Option<PathBuf>,
    rl: Editor<MyHelper, FileHistory>,
    history_file: PathBuf,
    spinner: Option<SpinnerHandle>,
}

///////////////////////////////////////////////////////////////////////////////
// FUNC
///////////////////////////////////////////////////////////////////////////////
fn clear_console() -> Result<()> {
    print!("{esc}c", esc = 27 as char);
    io::stdout().flush()?;
    Ok(())
}

fn init_readline(commands: &UserCommands) -> Result<(Editor<MyHelper, FileHistory>, PathBuf)> {
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .keyseq_timeout(Some(50))
        .build();

    let dirs = ProjectDirs::from(DIRS_QUALIFIER, DIRS_ORG, DIRS_APP).ok_or(Error::NotFound)?;

    let mut rl = Editor::with_config(config)?;

    let history_file = dirs.config_dir().join("history.txt");

    if !dirs.config_dir().exists() {
        fs::create_dir_all(dirs.config_dir())?;
    }

    if let Err(e) = rl.load_history(&history_file) {
        info!("loading history error={e}");
    }

    let mut command_names = Vec::new();
    for c in commands.list_commands() {
        command_names.push(c.name);
        for a in c.alias {
            command_names.push(a);
        }
    }

    let h = MyHelper {
        completer: CommandCompleter::new(command_names),
        hinter: HistoryHinter::new(),
    };

    rl.set_helper(Some(h));
    rl.bind_sequence(KeyEvent(KeyCode::Esc, Modifiers::NONE), Cmd::Kill(Movement::WholeLine));

    Ok((rl, history_file))
}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////
impl TerminalConsole {
    pub fn new(commands: &UserCommands) -> Result<Self> {
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

        // pretty start

        let (mut rl, history_file) = init_readline(commands)?;

        rl.clear_screen()?;

        let _ = display_banner(format!("{PKG_NAME} {PKG_VERSION}"), "pagga");

        Ok(Self {
            glow,
            rl,
            history_file,
            spinner: None,
        })
    }

    fn display_glow<S>(&self, glow: &Path, text: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        let mut child = std::process::Command::new(glow)
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
    }

    fn display_boring<S>(&self, text: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        print!("{}", text.as_ref());
        Ok(())
    }

    fn readline(&mut self) -> Result<String> {
        loop {
            match self.rl.readline("> ") {
                Ok(line) => {
                    if !line.trim().is_empty() {
                        let _ = self.rl.add_history_entry(&line);
                    }
                    return Ok(line);
                }
                // CTRL+D
                Err(ReadlineError::Eof) => return Err(Error::EOF),
                // CTRL+C
                Err(ReadlineError::Interrupted) => return Err(Error::EOF),
                Err(e) => {
                    error!("{e}");
                    return Err(e.into());
                }
            }
        }
    }

    pub async fn read_input(&mut self) -> Result<String> {
        loop {
            let line = self.readline()?;

            // remove leading / trailing white spaces
            let line = line.trim().to_string();

            if line.is_empty() {
                continue;
            }

            break Ok(line);
        }
    }

    fn display_usage<S>(&mut self, usage: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        self.display_string(usage)
    }

    fn display_md<M>(&mut self, data: M) -> Result<()>
    where
        M: AdoDataMarkdown,
    {
        let md = data.to_markdown()?;
        self.display_string(md)
    }

    fn display_base64<S>(&self, _data: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        unimplemented!()
    }

    pub fn display_error(&mut self, err: Error) -> Result<()> {
        match err {
            Error::LlmError { message } => self.display_string(message)?,
            _ => {
                let err_str = format!("Error: {err}");
                println!("{}", err_str.red());
            }
        }

        Ok(())
    }
}

impl Drop for TerminalConsole {
    fn drop(&mut self) {
        if let Err(e) = self.rl.save_history(&self.history_file) {
            error!("saving history error={e}");
        }
    }
}

impl ConsoleDisplayTrait for TerminalConsole {
    fn start_spinner(&mut self) {
        let spinner = SpinnerBuilder::new("".into()).start();
        self.spinner = Some(spinner)
    }
    fn stop_spinner(&mut self) {
        self.spinner = None
    }

    fn display<D>(&mut self, data: D) -> Result<()>
    where
        D: AsRef<AdoData>,
    {
        match data.as_ref() {
            AdoData::Empty => Ok(()),
            AdoData::Reset => clear_console(),
            AdoData::Json(s) => {
                let json_str = format!("```json\n{s}\n```");
                self.display_string(json_str)
            }
            AdoData::String(s) => self.display_string(s),
            AdoData::Base64(s) => self.display_base64(s),
            AdoData::SearchData(s) => self.display_md(s),
            AdoData::UsageString(s) => self.display_usage(s),
            AdoData::Shell(s) => self.display_md(s),
            AdoData::Status(s) => self.display_md(s),
            AdoData::LlmUsage(u) => self.display_md(u),
            AdoData::Bytes(_b) => todo!(),
        }
    }

    fn display_string<S>(&mut self, value: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        match &self.glow {
            Some(v) => self.display_glow(v, value),
            None => self.display_boring(value),
        }
    }
}
