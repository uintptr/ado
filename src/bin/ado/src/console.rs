use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Stdio,
};

use adolib::{
    const_vars::{
        DIRS_APP, DIRS_ORG, DIRS_QUALIFIER, PKG_NAME, PKG_VERSION, VERGEN_BUILD_DATE, VERGEN_RUSTC_COMMIT_HASH,
    },
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

use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{Completer, Helper, Hinter, Validator};
use rustyline::{CompletionType, Config, Editor};
use rustyline::{Highlighter, hint::HistoryHinter};

#[derive(Helper, Completer, Highlighter, Hinter, Validator)]
struct MyHelper {
    #[rustyline(Completer)]
    completer: FilenameCompleter,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
}

pub struct TerminalConsole {
    glow: Option<PathBuf>,
    rl: Editor<MyHelper, FileHistory>,
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

fn init_readline(commands: &UserCommands) -> Result<Editor<MyHelper, FileHistory>> {
    let config = Config::builder()
        .auto_add_history(true)
        .completion_type(CompletionType::List)
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

    let h = MyHelper {
        completer: FilenameCompleter::new(),
        hinter: HistoryHinter::new(),
    };

    for c in commands.list_commands() {
        if let Err(e) = rl.add_history_entry(c.name) {
            warn!("{e}");
        }

        for a in c.alias {
            if let Err(e) = rl.add_history_entry(a) {
                warn!("{e}");
            }
        }
    }

    rl.set_helper(Some(h));

    Ok(rl)
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
        clear_console()?;
        let banner = format!("{PKG_NAME} {PKG_VERSION} {VERGEN_RUSTC_COMMIT_HASH} ({VERGEN_BUILD_DATE})");
        println!("{}", banner.bold().yellow());

        Ok(Self {
            glow,
            rl: init_readline(commands)?,
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
            let res = match self.rl.readline("> ") {
                Ok(line) => {
                    self.rl.add_history_entry(&line)?;
                    Ok(line)
                }
                // CTRL+D
                Err(ReadlineError::Eof) => Err(Error::EOF),
                // CTRL+C
                Err(ReadlineError::Interrupted) => Err(Error::EOF),
                // ¯\_(ツ)_/¯
                Err(e) => {
                    error!("{e}");
                    Err(e.into())
                }
            };

            if let Ok(line) = &res {
                if line.is_empty() {
                    continue;
                }
                break res;
            } else {
                break res;
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
