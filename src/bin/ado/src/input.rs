use std::{borrow::Cow, env, fs, path::Path};

use anyhow::Result;
use crossterm::style::Stylize;
use reedline::{
    Completer, EditCommand, Emacs, FileBackedHistory, IdeMenu, KeyCode, KeyModifiers, MenuBuilder, Prompt,
    PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus, Reedline, ReedlineEvent, ReedlineMenu, Signal,
    Span, Suggestion, default_emacs_keybindings,
};

const MAX_SUGGESTIONS: u16 = 10;
// Over-collect before ranking so the best MAX_SUGGESTIONS bubble to the top
const COLLECT_LIMIT: usize = (MAX_SUGGESTIONS as usize).saturating_mul(4);
const HISTORY_CAPACITY: usize = 1000;

pub enum InputResult {
    Line(String),
    Eof,
}

pub struct AdoCompleter {
    commands: Vec<String>,
}

impl AdoCompleter {
    #[must_use]
    pub fn new(commands: Vec<String>) -> Self {
        Self { commands }
    }
}

fn command_description(name: &str) -> Option<String> {
    let desc = match name {
        "help" => "Show available commands",
        "model" => "Switch or show the current model",
        "models" => "List all available models",
        "reset" => "Clear the terminal screen",
        _ => return None,
    };
    Some(desc.to_string())
}

impl Completer for AdoCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        // @file completion
        if let Some((at_pos, partial)) = find_at_context(line, pos) {
            let files = find_matching_files(&partial);
            if !files.is_empty() {
                return files
                    .into_iter()
                    .map(|f| Suggestion {
                        value: format!("@{f}"),
                        description: None,
                        display_override: Some(f),
                        style: None,
                        extra: None,
                        span: Span::new(at_pos, pos),
                        append_whitespace: false,
                        match_indices: None,
                    })
                    .collect();
            }
        }

        // /command completion
        let safe_pos = (0..=pos.min(line.len())).rev().find(|&i| line.is_char_boundary(i)).unwrap_or(0);
        let before_cursor = &line[..safe_pos];
        if !before_cursor.contains(' ') && before_cursor.starts_with('/') {
            let partial = &before_cursor[1..];
            let matches = find_matching_commands(partial, &self.commands);
            if !matches.is_empty() {
                return matches
                    .into_iter()
                    .map(|cmd| {
                        let description = command_description(&cmd);
                        Suggestion {
                            value: format!("/{cmd}"),
                            description,
                            display_override: Some(cmd),
                            style: None,
                            extra: None,
                            span: Span::new(0, pos),
                            append_whitespace: true,
                            match_indices: None,
                        }
                    })
                    .collect();
            }
        }

        Vec::new()
    }
}

fn find_at_context(line: &str, cursor: usize) -> Option<(usize, String)> {
    let cursor = cursor.min(line.len());
    let cursor = (0..=cursor).rev().find(|&i| line.is_char_boundary(i))?;
    let before = &line[..cursor];

    let at_pos = before.rfind('@')?;

    let start = at_pos.saturating_add(1);
    let between = &before[start..];
    if between.contains(' ') {
        return None;
    }

    Some((at_pos, between.to_string()))
}

/// Returns relative paths from cwd. Directories have a trailing `/`.
fn find_matching_files(partial: &str) -> Vec<String> {
    let Ok(cwd) = env::current_dir() else { return Vec::new() };

    if partial.is_empty() {
        return list_top_level_entries(&cwd);
    }

    let partial_lower = partial.to_lowercase();
    let mut dirs: Vec<String> = Vec::new();
    let mut prefix_files: Vec<String> = Vec::new();
    let mut contains_files: Vec<String> = Vec::new();

    let walker = walkdir::WalkDir::new(&cwd)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'));

    for entry in walker.flatten() {
        let path = entry.path();
        let Ok(rel) = path.strip_prefix(&cwd) else { continue };
        if rel.as_os_str().is_empty() {
            continue;
        }

        let rel_str = rel.to_string_lossy().to_string();
        let rel_lower = rel_str.to_lowercase();

        if !rel_lower.contains(&partial_lower) {
            continue;
        }

        let is_dir = entry.file_type().is_dir();
        let display = if is_dir { format!("{rel_str}/") } else { rel_str };

        let total = dirs
            .len()
            .saturating_add(prefix_files.len())
            .saturating_add(contains_files.len());
        if total >= COLLECT_LIMIT {
            break;
        }

        if is_dir {
            dirs.push(display);
        } else if rel_lower.starts_with(&partial_lower) {
            prefix_files.push(display);
        } else {
            contains_files.push(display);
        }
    }

    dirs.into_iter()
        .chain(prefix_files)
        .chain(contains_files)
        .take(MAX_SUGGESTIONS as usize)
        .collect()
}

/// Lists the top-level entries of `cwd`: directories first (sorted), then files (sorted).
fn list_top_level_entries(cwd: &Path) -> Vec<String> {
    let Ok(read) = fs::read_dir(cwd) else { return Vec::new() };

    let mut dirs: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();

    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir {
            dirs.push(format!("{name}/"));
        } else {
            files.push(name);
        }
        if dirs.len().saturating_add(files.len()) >= COLLECT_LIMIT {
            break;
        }
    }

    dirs.sort();
    files.sort();

    dirs.into_iter().chain(files).take(MAX_SUGGESTIONS as usize).collect()
}

/// Case-insensitive prefix match, with contains fallback when no prefix matches.
fn find_matching_commands(partial: &str, commands: &[String]) -> Vec<String> {
    let partial_lower = partial.to_lowercase();

    let prefix: Vec<String> = commands
        .iter()
        .filter(|cmd| cmd.to_lowercase().starts_with(&partial_lower))
        .cloned()
        .collect();

    if !prefix.is_empty() {
        return prefix;
    }

    // fallback: contains match
    commands
        .iter()
        .filter(|cmd| cmd.to_lowercase().contains(&partial_lower))
        .cloned()
        .collect()
}

fn build_editor(history_file: &Path, commands: Vec<String>) -> Result<Reedline> {
    let completer = Box::new(AdoCompleter::new(commands));

    let completion_menu = Box::new(
        IdeMenu::default()
            .with_name("completion_menu")
            .with_min_completion_width(20)
            .with_max_completion_height(MAX_SUGGESTIONS),
    );

    let mut keybindings = default_emacs_keybindings();
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );

    // ESC clears the current input buffer
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Esc,
        ReedlineEvent::Edit(vec![EditCommand::SelectAll, EditCommand::CutToEnd]),
    );

    // Auto-trigger completion on '/' and '@'
    for c in ['/', '@'] {
        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Char(c),
            ReedlineEvent::Multiple(vec![
                ReedlineEvent::Edit(vec![EditCommand::InsertChar(c)]),
                ReedlineEvent::Menu("completion_menu".to_string()),
            ]),
        );
    }

    let edit_mode = Box::new(Emacs::new(keybindings));

    let history = Box::new(FileBackedHistory::with_file(
        HISTORY_CAPACITY,
        history_file.to_path_buf(),
    )?);

    let editor = Reedline::create()
        .with_completer(completer)
        .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
        .with_edit_mode(edit_mode)
        .with_quick_completions(true)
        .with_partial_completions(true)
        .with_history(history)
        .use_bracketed_paste(true);

    Ok(editor)
}

pub fn create_editor(history_file: &Path, commands: Vec<String>) -> Result<Reedline> {
    build_editor(history_file, commands)
}

pub struct AdoPrompt {
    model: String,
}

impl AdoPrompt {
    pub fn new(model: impl Into<String>) -> Self {
        Self { model: model.into() }
    }
}

impl Prompt for AdoPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Owned(format!("{}", format!("[{}]", self.model).dark_grey()))
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Owned(format!(" {} ", "❯".green()))
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed(": ")
    }

    fn render_prompt_history_search_indicator(&self, history_search: PromptHistorySearch) -> Cow<'_, str> {
        match history_search.status {
            PromptHistorySearchStatus::Passing => Cow::Borrowed("(search): "),
            PromptHistorySearchStatus::Failing => Cow::Borrowed("(failed): "),
        }
    }
}

pub fn read_line(editor: &mut Reedline, model: &str) -> Result<InputResult> {
    let prompt = AdoPrompt::new(model);
    match editor.read_line(&prompt) {
        Ok(Signal::Success(line)) => Ok(InputResult::Line(line)),
        Ok(Signal::CtrlC | Signal::CtrlD) => Ok(InputResult::Eof),
        Err(e) => Err(e.into()),
    }
}
