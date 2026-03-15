use std::{borrow::Cow, env, path::Path};

use anyhow::Result;
use reedline::{
    Completer, EditCommand, FileBackedHistory, IdeMenu, KeyCode, KeyModifiers, MenuBuilder,
    Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus, Reedline, ReedlineEvent,
    ReedlineMenu, Signal, Span, Suggestion, default_emacs_keybindings, Emacs,
};

const MAX_SUGGESTIONS: u16 = 7;
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

impl Completer for AdoCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        // Check for @file completion
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

        // Check for /command completion
        let safe_pos = (0..=pos.min(line.len())).rev().find(|&i| line.is_char_boundary(i)).unwrap_or(0);
        let before_cursor = &line[..safe_pos];
        if !before_cursor.contains(' ') && before_cursor.starts_with('/') {
            let partial = &before_cursor[1..];
            let matches = find_matching_commands(partial, &self.commands);
            if !matches.is_empty() {
                return matches
                    .into_iter()
                    .map(|cmd| Suggestion {
                        value: format!("/{cmd}"),
                        description: None,
                        display_override: Some(cmd),
                        style: None,
                        extra: None,
                        span: Span::new(0, pos),
                        append_whitespace: true,
                        match_indices: None,
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

fn find_matching_files(partial: &str) -> Vec<String> {
    if partial.is_empty() {
        return Vec::new();
    }

    let Ok(cwd) = env::current_dir() else { return Vec::new() };

    let pattern = format!("{}/**/*", cwd.display());
    let partial_lower = partial.to_lowercase();
    let mut results = Vec::new();

    let Ok(entries) = glob::glob(&pattern) else {
        return Vec::new();
    };

    for entry in entries.flatten() {
        let Ok(rel) = entry.strip_prefix(&cwd) else { continue };

        if rel.components().any(|c| c.as_os_str().to_string_lossy().starts_with('.')) {
            continue;
        }

        let display = rel.to_string_lossy().to_string();
        if !display.to_lowercase().contains(&partial_lower) {
            continue;
        }

        results.push(display);

        if results.len() >= MAX_SUGGESTIONS as usize {
            break;
        }
    }

    results
}

fn find_matching_commands(partial: &str, commands: &[String]) -> Vec<String> {
    commands.iter().filter(|cmd| cmd.starts_with(partial)).cloned().collect()
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

    // Auto-trigger completion menu only on '/' and '@' (the trigger chars)
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

    let history = Box::new(FileBackedHistory::with_file(HISTORY_CAPACITY, history_file.to_path_buf())?);

    let editor = Reedline::create()
        .with_completer(completer)
        .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
        .with_edit_mode(edit_mode)
        .with_quick_completions(true)
        .with_partial_completions(true)
        .with_history(history);

    Ok(editor)
}

pub fn create_editor(history_file: &Path, commands: Vec<String>) -> Result<Reedline> {
    build_editor(history_file, commands)
}

struct AdoPrompt;

impl Prompt for AdoPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }
    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }
    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Borrowed("> ")
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

pub fn read_line(editor: &mut Reedline) -> Result<InputResult> {
    match editor.read_line(&AdoPrompt) {
        Ok(Signal::Success(line)) => Ok(InputResult::Line(line)),
        Ok(Signal::CtrlC | Signal::CtrlD) => Ok(InputResult::Eof),
        Err(e) => Err(e.into()),
    }
}
