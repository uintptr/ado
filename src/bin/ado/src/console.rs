use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Stdio,
};

use adolib::{
    const_vars::{DOT_DIRECTORY, PKG_NAME, PKG_VERSION, VERGEN_BUILD_DATE, VERGEN_RUSTC_COMMIT_HASH},
    data::types::{AdoData, AdoDataMarkdown},
    error::{Error, Result},
    ui::{ConsoleDisplayTrait, commands::UserCommands},
};
use colored;
use colored::Colorize;
use log::{error, info, warn};
use spinner::{SpinnerBuilder, SpinnerHandle};
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

    let mut rl = Editor::with_config(config)?;

    let home = env::var("HOME")?;

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

///////////////////////////////////////////////////////////////////////////////
// TEST
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {

    use std::{fs, path::Path};

    use adolib::{
        config::loader::AdoConfig, data::types::AdoData, logging::logger::setup_logger,
        search::google::GoogleSearchResults, shell::AdoShell, ui::commands::UserCommands,
    };

    use adolib::storage::persistent::PersistentStorage;

    use crate::console::TerminalConsole;
    use adolib::ui::{ConsoleDisplayTrait, NopConsole};

    #[test]
    fn display_text() {
        setup_logger(true).unwrap();

        let config = AdoConfig::from_default().unwrap();

        let td = tempfile::Builder::new().prefix("console_test_").tempdir().unwrap();
        let cache_file = td.path().join("cache.db");
        let cache = PersistentStorage::from_path(cache_file).unwrap();
        let command = UserCommands::new(&config, cache).unwrap();
        let mut console = TerminalConsole::new(&command).unwrap();
        console.display(AdoData::String("Hello, World!".to_string())).unwrap();
    }

    #[tokio::test]
    async fn arg_parser() {
        setup_logger(true).unwrap();

        let config = AdoConfig::from_default().unwrap();

        let td = tempfile::Builder::new().prefix("console_test_").tempdir().unwrap();
        let cache_file = td.path().join("cache.db");
        let cache = PersistentStorage::from_path(cache_file).unwrap();

        let mut cmd = UserCommands::new(&config, cache).unwrap();

        let mut console = NopConsole {};

        cmd.handler("/quit", &mut console).await.unwrap();
    }

    #[test]
    fn display_search() {
        setup_logger(true).unwrap();

        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let json_file = Path::new(manifest_dir)
            .join("..")
            .join("..")
            .join("..")
            .join("test")
            .join("search_test.json")
            .canonicalize()
            .unwrap();

        let json_data = fs::read_to_string(json_file).unwrap();

        let config = AdoConfig::from_default().unwrap();

        let td = tempfile::Builder::new().prefix("console_test_").tempdir().unwrap();
        let cache_file = td.path().join("cache.db");
        let cache = PersistentStorage::from_path(cache_file).unwrap();
        let command = UserCommands::new(&config, cache).unwrap();
        let mut console = TerminalConsole::new(&command).unwrap();

        let data = AdoData::SearchData(GoogleSearchResults { json_string: json_data });

        console.display(data).unwrap();
    }

    #[test]
    fn display_shell_test() {
        setup_logger(true).unwrap();

        let sh = AdoShell::new();

        let data = sh.exec("uname -a").unwrap();

        let config = AdoConfig::from_default().unwrap();

        let td = tempfile::Builder::new().prefix("console_test_").tempdir().unwrap();
        let cache_file = td.path().join("cache.db");
        let cache = PersistentStorage::from_path(cache_file).unwrap();

        let command = UserCommands::new(&config, cache).unwrap();

        let mut console = TerminalConsole::new(&command).unwrap();

        console.display(data).unwrap();
    }
}
