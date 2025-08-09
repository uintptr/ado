use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Stdio,
};

use adolib::{
    config_file::loader::ConfigFile,
    const_vars::{DOT_DIRECTORY, PKG_NAME, PKG_VERSION, VERGEN_BUILD_DATE, VERGEN_RUSTC_COMMIT_HASH},
    data::types::{AdoData, AdoDataMarkdown},
    error::{Error, Result},
    ui::commands::UserCommands,
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
        let banner = format!("{PKG_NAME} {PKG_VERSION} {VERGEN_RUSTC_COMMIT_HASH} ({VERGEN_BUILD_DATE})");
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

    pub fn display_string<S>(&self, value: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        match &self.glow {
            Some(v) => self.display_glow(v, value),
            None => self.display_boring(value),
        }
    }

    fn display_usage<S>(&self, usage: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        self.display_string(usage)
    }

    fn display_md<M>(&self, data: M) -> Result<()>
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

    fn display(&self, data: AdoData) -> Result<()> {
        match data {
            AdoData::Empty => Ok(()),
            AdoData::Reset => clear_console(),
            AdoData::Json(s) => self.display_md(s),
            AdoData::String(s) => self.display_md(s),
            AdoData::Base64(s) => self.display_base64(s),
            AdoData::Http(s) => self.display_md(s),
            AdoData::SearchData(s) => self.display_md(s),
            AdoData::UsageString(s) => self.display_usage(s),
            AdoData::Shell(s) => self.display_md(s),
            AdoData::Status(s) => self.display_md(s),
            _ => unimplemented!(),
        }
    }

    pub fn display_messages(&self, data: AdoData) -> Result<()> {
        self.display(data)
    }

    pub fn display_error(&self, err: Error) -> Result<()> {
        let err_str = format!("Error: {err}");
        println!("{}", err_str.red());
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::{fs, path::Path};

    use adolib::{
        config_file::loader::ConfigFile,
        data::{search::GoogleSearchData, types::AdoData},
        logging::logger::setup_logger,
        shell::AdoShell,
        ui::commands::UserCommands,
    };

    use super::ConsoleUI;

    #[test]
    fn display_text() {
        setup_logger(true).unwrap();

        let config = ConfigFile::from_disk().unwrap();

        let console = ConsoleUI::new(&config).unwrap();

        console.display(AdoData::String("Hello, World!".to_string())).unwrap();
    }

    #[tokio::test]
    async fn arg_parser() {
        setup_logger(true).unwrap();

        let config = ConfigFile::from_disk().unwrap();

        let mut cmd = UserCommands::new(&config).unwrap();

        cmd.handler("/quit").await.unwrap();
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

        let config = ConfigFile::from_disk().unwrap();
        let console = ConsoleUI::new(&config).unwrap();

        let data = AdoData::SearchData(GoogleSearchData { json_string: json_data });

        console.display(data).unwrap();
    }

    #[test]
    fn display_shell_test() {
        setup_logger(true).unwrap();

        let sh = AdoShell::new();

        let data = sh.exec("uname -a").unwrap();

        let config = ConfigFile::from_disk().unwrap();
        let console = ConsoleUI::new(&config).unwrap();

        console.display(data).unwrap();
    }
}
