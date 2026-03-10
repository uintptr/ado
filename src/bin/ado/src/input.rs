use std::{
    env,
    io::{self, stdout},
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    ExecutableCommand, cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    terminal,
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget},
};
use tui_input::backend::crossterm::EventHandler;
use tui_input::{Input, InputRequest};

const MAX_SUGGESTIONS: usize = 7;

pub enum InputResult {
    Line(String),
    Eof,
}

struct InputState {
    input: Input,
    history_index: Option<usize>,
    saved_input: String,
    suggestions: Vec<String>,
    suggestion_index: usize,
    show_popup: bool,
    at_start: Option<usize>,
}

impl InputState {
    fn new() -> Self {
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

    fn value(&self) -> &str {
        self.input.value()
    }

    fn cursor(&self) -> usize {
        self.input.cursor()
    }
}

// Find the @token context around the cursor
fn find_at_context(line: &str, cursor: usize) -> Option<(usize, String)> {
    let cursor = cursor.min(line.len());
    // Walk back to a valid char boundary
    let cursor = (0..=cursor).rev().find(|&i| line.is_char_boundary(i))?;
    let before = &line[..cursor];

    // Scan backwards for @
    let at_pos = before.rfind('@')?;

    // No whitespace between @ and cursor
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

    // Treat input as a filename pattern: search **/* then filter case-insensitively
    let pattern = format!("{}/**/*", cwd.display());
    let partial_lower = partial.to_lowercase();
    let mut results = Vec::new();

    let Ok(entries) = glob::glob(&pattern) else {
        return Vec::new();
    };

    for entry in entries.flatten() {
        // Skip hidden paths (any component starting with .)
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

fn find_matching_commands(partial: &str, commands: &[String]) -> Vec<String> {
    commands.iter().filter(|cmd| cmd.starts_with(partial)).cloned().collect()
}

fn update_suggestions(state: &mut InputState, commands: &[String]) {
    let line = state.value().to_string();
    let cursor = state.cursor();

    // Check for @file context
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

    // Check for command completion (first word starting with /)
    let before_cursor = &line[..cursor];
    if !before_cursor.contains(' ') && before_cursor.starts_with('/') {
        let partial = &before_cursor[1..]; // strip /
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

fn accept_suggestion(state: &mut InputState) {
    if !state.show_popup || state.suggestions.is_empty() {
        return;
    }

    let Some(suggestion) = state.suggestions.get(state.suggestion_index).cloned() else {
        return;
    };

    if let Some(at_pos) = state.at_start {
        // @file completion: replace @partial with @suggestion
        let line = state.value().to_string();
        let cursor = state.cursor();
        let before_at = &line[..at_pos];
        let after_cursor = &line[cursor..];
        let new_line = format!("{before_at}@{suggestion}{after_cursor}");
        let new_cursor = at_pos.saturating_add(1).saturating_add(suggestion.len());

        state.input = Input::new(new_line);
        // Move cursor to correct position
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
        // Command completion: replace entire input
        let new_line = format!("/{suggestion}");
        state.input = Input::new(new_line);
    }

    state.suggestions.clear();
    state.show_popup = false;
    state.at_start = None;
}

fn history_prev(state: &mut InputState, history: &[String]) {
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

fn history_next(state: &mut InputState, history: &[String]) {
    match state.history_index {
        None => (),
        Some(i) => {
            let next = i.saturating_add(1);
            if next >= history.len() {
                // Restore saved input
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

pub fn read_line(prompt: &str, history: &mut [String], commands: &[String]) -> Result<InputResult> {
    terminal::enable_raw_mode()?;

    let mut stdout = stdout();
    stdout.execute(cursor::Show)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::with_options(
        backend,
        ratatui::TerminalOptions {
            viewport: ratatui::Viewport::Inline(u16::try_from(MAX_SUGGESTIONS).unwrap_or(u16::MAX).saturating_add(2)),
        },
    )?;

    let mut state = InputState::new();
    let result = run_event_loop(&mut terminal, &mut state, prompt, history, commands);

    // Push the completed prompt+input into scrollback before closing
    if let Ok(InputResult::Line(ref line)) = result {
        let prompt_line = Line::from(vec![
            Span::styled(prompt, Style::default().fg(Color::Green)),
            Span::raw(line.as_str()),
        ]);
        terminal.insert_before(1, |buf| {
            Paragraph::new(prompt_line).render(buf.area, buf);
        })?;
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(cursor::Show)?;

    result
}

fn to_u16(val: usize) -> u16 {
    u16::try_from(val).unwrap_or(u16::MAX)
}

fn draw_frame(
    frame: &mut ratatui::Frame,
    state: &InputState,
    prompt: &str,
    popup_height: u16,
) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(popup_height),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(area);

    let Some(popup_area) = chunks.first().copied() else { return };
    let Some(input_area) = chunks.get(1).copied() else { return };

    if state.show_popup && !state.suggestions.is_empty() {
        let items: Vec<ListItem> = state
            .suggestions
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let style = if i == state.suggestion_index {
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };
                ListItem::new(Line::from(Span::styled(s.clone(), style)))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        frame.render_widget(Clear, popup_area);
        frame.render_widget(list, popup_area);
    }

    let cursor_pos = to_u16(state.cursor()).saturating_add(to_u16(prompt.len()));
    let input_line = Line::from(vec![
        Span::styled(prompt, Style::default().fg(Color::Green)),
        Span::raw(state.value()),
    ]);

    frame.render_widget(Paragraph::new(input_line), input_area);
    frame.set_cursor_position((input_area.x.saturating_add(cursor_pos), input_area.y));
}

fn handle_key(state: &mut InputState, key: event::KeyEvent, history: &[String], commands: &[String]) -> Option<InputResult> {
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

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut InputState,
    prompt: &str,
    history: &[String],
    commands: &[String],
) -> Result<InputResult> {
    loop {
        let popup_height = if state.show_popup {
            to_u16(state.suggestions.len().min(MAX_SUGGESTIONS)).saturating_add(2)
        } else {
            0
        };

        terminal.draw(|frame| {
            draw_frame(frame, state, prompt, popup_height);
        })?;

        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
            && let Some(result) = handle_key(state, key, history, commands)
        {
            return Ok(result);
        }
    }
}
