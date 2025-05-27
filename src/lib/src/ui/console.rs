use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use log::{error, info, warn};
use which::which;

use crate::{
    const_vars::DOT_DIRECTORY,
    error::{Error, Result},
};

use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Completer, Helper, Hinter, Validator};
use rustyline::{CompletionType, Config, EditMode, Editor};
use rustyline::{Highlighter, hint::HistoryHinter};

use super::{UiTrait, user_commands::UserCommands};

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
    commands: UserCommands,
}

fn clear_console() -> Result<()> {
    print!("{esc}c", esc = 27 as char);
    io::stdout().flush()?;
    Ok(())
}

fn init_readline() -> Result<Editor<MyHelper, FileHistory>> {
    let config = Config::builder()
        .auto_add_history(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Vi)
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

    rl.set_helper(Some(h));

    Ok(rl)
}

impl ConsoleUI {
    pub fn new() -> Result<Self> {
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

        Ok(Self {
            glow,
            rl: init_readline()?,
            commands: UserCommands::new(),
        })
    }

    fn display_glow(&self, glow: &Path, text: &str) -> Result<()> {
        let mut child = Command::new(glow).stdin(Stdio::piped()).spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(text.as_bytes())?;
        }

        child.wait()?;

        Ok(())
    }

    fn display_boring(&self, text: &str) -> Result<()> {
        print!("{text}");
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
}

impl UiTrait for ConsoleUI {
    fn display(&self, text: &str) -> Result<()> {
        match &self.glow {
            Some(v) => self.display_glow(v, text),
            None => self.display_boring(text),
        }
    }

    fn read_input(&mut self) -> Result<String> {
        loop {
            let line = self.readline()?;

            // remove leading / trailing white spaces
            let line = line.trim().to_string();

            match self.commands.handler(&line) {
                Ok(r) => {
                    self.display(&r)?;
                    continue;
                }
                Err(Error::CommandNotFound) => break Ok(line),
                Err(e) => break Err(e.into()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::staples::setup_logger;

    use super::ConsoleUI;

    #[test]
    fn display_text() {
        setup_logger(true).unwrap();

        let console = ConsoleUI::new();

        console.display_text("hello world").unwrap();
    }
}
