use std::{
    env,
    io::{self, Write},
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    cursor, execute, queue,
    event::{self, Event, KeyCode, KeyModifiers},
    style::{self, Attribute},
    terminal::{self, ClearType},
};
use tui_input::backend::crossterm::EventHandler;
use tui_input::{Input, InputRequest};

pub(crate) const MAX_SUGGESTIONS: usize = 7;

pub enum InputResult {
    Line(String),
    Eof,
}

pub(crate) struct InputState {
    pub(crate) input: Input,
    pub(crate) history_index: Option<usize>,
    pub(crate) saved_input: String,
    pub(crate) suggestions: Vec<String>,
    pub(crate) suggestion_index: usize,
    pub(crate) show_popup: bool,
    pub(crate) at_start: Option<usize>,
}

impl InputState {
    pub(crate) fn new() -> Self {
        Self {
            input: Input::default(),
            history_index: None,
            saved_input: String::new(),
            suggestions: Vec::new(),
            suggestion_index: 0,
            show_popup: false,
            at_start: None,
        }
    }

    pub(crate) fn value(&self) -> &str {
        self.input.value()
    }

    pub(crate) fn cursor(&self) -> usize {
        self.input.cursor()
    }
}

// Find the @token context around the cursor
pub(crate) fn find_at_context(line: &str, cursor: usize) -> Option<(usize, String)> {
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

pub(crate) fn find_matching_files(partial: &str) -> Vec<String> {
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

        if results.len() >= MAX_SUGGESTIONS {
            break;
        }
    }

    results
}

pub(crate) fn find_matching_commands(partial: &str, commands: &[String]) -> Vec<String> {
    commands.iter().filter(|cmd| cmd.starts_with(partial)).cloned().collect()
}

pub(crate) fn update_suggestions(state: &mut InputState, commands: &[String]) {
    let line = state.value().to_string();
    let cursor = state.cursor();

    if let Some((at_pos, partial)) = find_at_context(&line, cursor) {
        let files = find_matching_files(&partial);
        if !files.is_empty() {
            state.suggestions = files;
            state.suggestion_index = 0;
            state.show_popup = true;
            state.at_start = Some(at_pos);
            return;
        }
    }

    let before_cursor = &line[..cursor];
    if !before_cursor.contains(' ') && before_cursor.starts_with('/') {
        let partial = &before_cursor[1..];
        let matches = find_matching_commands(partial, commands);
        if !matches.is_empty() {
            state.suggestions = matches;
            state.suggestion_index = 0;
            state.show_popup = true;
            state.at_start = None;
            return;
        }
    }

    state.suggestions.clear();
    state.show_popup = false;
    state.at_start = None;
}

pub(crate) fn accept_suggestion(state: &mut InputState) {
    if !state.show_popup || state.suggestions.is_empty() {
        return;
    }

    let Some(suggestion) = state.suggestions.get(state.suggestion_index).cloned() else {
        return;
    };

    if let Some(at_pos) = state.at_start {
        let line = state.value().to_string();
        let cursor = state.cursor();
        let before_at = &line[..at_pos];
        let after_cursor = &line[cursor..];
        let new_line = format!("{before_at}@{suggestion}{after_cursor}");
        let new_cursor = at_pos.saturating_add(1).saturating_add(suggestion.len());

        state.input = Input::new(new_line);
        let current = state.input.cursor();
        if new_cursor < current {
            for _ in 0..current.saturating_sub(new_cursor) {
                state.input.handle(InputRequest::GoToPrevChar);
            }
        } else {
            for _ in 0..new_cursor.saturating_sub(current) {
                state.input.handle(InputRequest::GoToNextChar);
            }
        }
    } else {
        let new_line = format!("/{suggestion}");
        state.input = Input::new(new_line);
    }

    state.suggestions.clear();
    state.show_popup = false;
    state.at_start = None;
}

pub(crate) fn history_prev(state: &mut InputState, history: &[String]) {
    if history.is_empty() {
        return;
    }

    match state.history_index {
        None => {
            state.saved_input = state.value().to_string();
            state.history_index = Some(history.len().saturating_sub(1));
        }
        Some(0) => return,
        Some(i) => {
            state.history_index = Some(i.saturating_sub(1));
        }
    }

    if let Some(idx) = state.history_index
        && let Some(entry) = history.get(idx)
    {
        state.input = Input::new(entry.clone());
    }
}

pub(crate) fn history_next(state: &mut InputState, history: &[String]) {
    match state.history_index {
        None => (),
        Some(i) => {
            let next = i.saturating_add(1);
            if next >= history.len() {
                state.history_index = None;
                let saved = state.saved_input.clone();
                state.input = Input::new(saved);
            } else {
                state.history_index = Some(next);
                if let Some(entry) = history.get(next) {
                    state.input = Input::new(entry.clone());
                }
            }
        }
    }
}

/// Render the prompt, input value, and any suggestions below.
/// Returns the number of suggestion lines rendered below the input.
fn render(stdout: &mut io::Stdout, state: &InputState, prompt: &str, prev_popup_lines: usize) -> io::Result<usize> {
    // Move back up to clear previous popup lines
    if prev_popup_lines > 0 {
        // We're currently at the end of the last popup line (or input line if popup was cleared).
        // Move up past all popup lines to get back to the input line.
        queue!(stdout, cursor::MoveUp(u16::try_from(prev_popup_lines).unwrap_or(u16::MAX)))?;
    }

    // Clear from input line down (clears old popup too)
    queue!(
        stdout,
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::FromCursorDown),
    )?;

    // Print prompt + input value
    queue!(
        stdout,
        style::SetForegroundColor(style::Color::Green),
        style::Print(prompt),
        style::ResetColor,
        style::Print(state.value()),
    )?;

    // Render suggestions below
    let popup_lines = if state.show_popup && !state.suggestions.is_empty() {
        let count = state.suggestions.len().min(MAX_SUGGESTIONS);
        for (i, s) in state.suggestions.iter().take(count).enumerate() {
            if i == state.suggestion_index {
                queue!(
                    stdout,
                    style::Print("\n"),
                    style::SetBackgroundColor(style::Color::DarkGrey),
                    style::SetForegroundColor(style::Color::White),
                    style::SetAttribute(Attribute::Bold),
                    style::Print(format!("  {s}")),
                    style::SetAttribute(Attribute::Reset),
                    style::ResetColor,
                )?;
            } else {
                queue!(
                    stdout,
                    style::Print("\n"),
                    style::SetForegroundColor(style::Color::Grey),
                    style::Print(format!("  {s}")),
                    style::ResetColor,
                )?;
            }
        }
        count
    } else {
        0
    };

    // Move cursor back to the input line at the correct column
    if popup_lines > 0 {
        queue!(stdout, cursor::MoveUp(u16::try_from(popup_lines).unwrap_or(u16::MAX)))?;
    }
    let cursor_col = prompt.len().saturating_add(state.cursor());
    queue!(stdout, cursor::MoveToColumn(u16::try_from(cursor_col).unwrap_or(u16::MAX)))?;

    stdout.flush()?;
    Ok(popup_lines)
}

fn handle_key(
    state: &mut InputState,
    key: event::KeyEvent,
    history: &[String],
    commands: &[String],
) -> Option<InputResult> {
    if key.kind == event::KeyEventKind::Release {
        return None;
    }

    match key.code {
        KeyCode::Enter => {
            if state.show_popup && !state.suggestions.is_empty() {
                accept_suggestion(state);
                update_suggestions(state, commands);
            } else {
                let line = state.value().to_string();
                return Some(InputResult::Line(line));
            }
        }
        KeyCode::Tab => {
            if state.show_popup {
                accept_suggestion(state);
                update_suggestions(state, commands);
            }
        }
        KeyCode::Up => {
            if state.show_popup {
                if state.suggestion_index > 0 {
                    state.suggestion_index = state.suggestion_index.saturating_sub(1);
                }
            } else {
                history_prev(state, history);
            }
        }
        KeyCode::Down => {
            if state.show_popup {
                if state.suggestion_index.saturating_add(1) < state.suggestions.len() {
                    state.suggestion_index = state.suggestion_index.saturating_add(1);
                }
            } else {
                history_next(state, history);
            }
        }
        KeyCode::Esc => {
            if state.show_popup {
                state.show_popup = false;
                state.suggestions.clear();
            } else {
                state.input = Input::default();
            }
        }
        KeyCode::Char('d' | 'c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(InputResult::Eof);
        }
        _ => {
            state.input.handle_event(&Event::Key(key));
            update_suggestions(state, commands);
        }
    }
    None
}

pub fn read_line(prompt: &str, history: &mut [String], commands: &[String]) -> Result<InputResult> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Show)?;

    let mut state = InputState::new();
    let mut prev_popup_lines = 0;

    prev_popup_lines = render(&mut stdout, &state, prompt, prev_popup_lines)?;

    let result = loop {
        if !event::poll(Duration::from_millis(50))? {
            continue;
        }
        if let Event::Key(key) = event::read()? {
            if let Some(result) = handle_key(&mut state, key, history, commands) {
                break result;
            }
            prev_popup_lines = render(&mut stdout, &state, prompt, prev_popup_lines)?;
        }
    };

    // Clean up: clear popup lines, move to a fresh line
    if prev_popup_lines > 0 {
        // Already on input line; clear popup below
        queue!(
            stdout,
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::FromCursorDown),
        )?;
    }
    // Move to start of next line
    execute!(stdout, style::Print("\r\n"))?;

    terminal::disable_raw_mode()?;

    Ok(result)
}
