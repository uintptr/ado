use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Stdio,
};

use adolib::{
    config::file::ConfigFile,
    const_vars::{DOT_DIRECTORY, PKG_NAME, PKG_VERSION},
    data::AdoData,
    error::{Error, Result},
    ui::commands::{Command, CommandResponse, UserCommands},
};
use colored;
use colored::Colorize;
use log::{error, info, warn};
use which::which;

use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Completer, Helper, Hinter, Validator};
use rustyline::{CompletionType, Config, Editor};
use rustyline::{Highlighter, hint::HistoryHinter};

#[derive(Helper, Completer, Highlighter, Hinter, Validator)]
struct MyHelper {
    #[rustyline(Completer)]
    completer: FilenameCompleter,
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
}

pub struct ConsoleUI {
    glow: Option<PathBuf>,
    rl: Editor<MyHelper, FileHistory>,
}

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

    let mut rl = Editor::with_config(config)?;

    let home = home::home_dir().ok_or(Error::HomeDirNotFound)?;

    let dot_dir = Path::new(&home).join(DOT_DIRECTORY);

    let history_file = Path::new(&dot_dir).join("history.txt");

    if !dot_dir.exists() {
        fs::create_dir_all(dot_dir)?;
    }

    if let Err(e) = rl.load_history(&history_file) {
        info!("loading history error={e}");
    }

    let h = MyHelper {
        completer: FilenameCompleter::new(),
        hinter: HistoryHinter::new(),
        validator: MatchingBracketValidator::new(),
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

impl ConsoleUI {
    pub fn new(config: &ConfigFile) -> Result<Self> {
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

        let commands = UserCommands::new(config)?;

        // pretty start
        clear_console()?;
        let banner = format!("{} {}", PKG_NAME, PKG_VERSION);
        println!("{}", banner.bold().yellow());

        Ok(Self {
            glow,
            rl: init_readline(&commands)?,
        })
    }

    fn display_glow<S>(&self, glow: &Path, text: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        let mut child = std::process::Command::new(glow).stdin(Stdio::piped()).spawn()?;

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

    pub fn display(&self, data: &AdoData) -> Result<()> {
        let msg = match data {
            AdoData::Json(s) => &format!("```json\n{s}\n```"),
            AdoData::String(s) => s,
            AdoData::Base64(s) => s,
            AdoData::Bytes(_) => &data.to_base64()?,
            AdoData::Http(s) => &serde_json::to_string(s)?,
        };

        match &self.glow {
            Some(v) => self.display_glow(v, msg),
            None => self.display_boring(msg),
        }
    }

    pub fn display_search(&self, data: &AdoData) -> Result<()> {
        //
        // display nicely
        //
        self.display(data)
    }

    pub fn display_messages(&self, resp: &CommandResponse) -> Result<()> {
        if let Some(data) = &resp.data {
            match &resp.command {
                Command::Search { query: _ } => self.display_search(data)?,
                _ => self.display(data)?,
            }
        }

        Ok(())
    }

    pub fn display_error(&self, err: Error) -> Result<()> {
        let err_str = format!("Error: {err}");
        println!("{}", err_str.red());
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use adolib::{config::file::ConfigFile, data::AdoData, logging::logger::setup_logger, ui::commands::UserCommands};

    use super::ConsoleUI;

    #[test]
    fn display_text() {
        setup_logger(true).unwrap();

        let config = ConfigFile::load().unwrap();

        let console = ConsoleUI::new(&config).unwrap();

        console.display(&AdoData::String("Hello, World!".to_string())).unwrap();
    }

    #[tokio::test]
    async fn arg_parser() {
        setup_logger(true).unwrap();

        let config = ConfigFile::load().unwrap();

        let mut cmd = UserCommands::new(&config).unwrap();

        cmd.handler("/quit").await.unwrap();
    }
}
