use std::{
    fs,
    io::{self, Stderr},
    path::PathBuf,
    time::Duration,
};

pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

use adolib::{
    const_vars::{DIRS_APP, DIRS_ORG, DIRS_QUALIFIER},
    data::types::{AdoData, AdoDataMarkdown},
    error::{Error, Result},
    ui::{ConsoleDisplayTrait, commands::UserCommands},
};
use directories::ProjectDirs;
use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use tui_textarea::TextArea;

use crate::banner::generate_banner;

const SPINNER_CHARS: [char; 8] = ['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷'];

enum InputAction {
    Submit(String),
    Quit,
}

pub struct TerminalConsole {
    terminal: Terminal<CrosstermBackend<Stderr>>,
    output_lines: Vec<Text<'static>>,
    scroll_offset: u16,
    output_area_height: u16,
    spinner_active: bool,
    spinner_frame: usize,
    input: TextArea<'static>,
    banner: Text<'static>,
    command_names: Vec<String>,
    history: Vec<String>,
    history_index: Option<usize>,
    history_file: PathBuf,
}

///////////////////////////////////////////////////////////////////////////////
// HELPERS
///////////////////////////////////////////////////////////////////////////////

fn load_history(path: &PathBuf) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

fn save_history(path: &PathBuf, history: &[String]) {
    let content = history.join("\n");
    let _ = fs::write(path, content);
}

fn create_input() -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_block(Block::default().borders(Borders::TOP).title(" > "));
    textarea
}

/// Simple markdown-to-styled-text converter.
///
/// Handles headings, code blocks (with syntect highlighting), bold, and plain text.
/// Produces ratatui `Text<'static>` for display in a Paragraph widget.
fn render_markdown(input: &str) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_buf: Vec<String> = Vec::new();

    for raw_line in input.lines() {
        if raw_line.starts_with("```") {
            if in_code_block {
                // Closing fence — highlight collected code
                let highlighted = highlight_code_block(&code_buf, &code_lang);
                lines.extend(highlighted);
                code_buf.clear();
                code_lang.clear();
                in_code_block = false;
            } else {
                // Opening fence — extract language hint
                code_lang = raw_line.trim_start_matches('`').trim().to_string();
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            code_buf.push(raw_line.to_string());
            continue;
        }

        if let Some(heading) = raw_line.strip_prefix("### ") {
            lines.push(Line::styled(
                heading.to_string(),
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            ));
        } else if let Some(heading) = raw_line.strip_prefix("## ") {
            lines.push(Line::styled(
                heading.to_string(),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ));
        } else if let Some(heading) = raw_line.strip_prefix("# ") {
            lines.push(Line::styled(
                heading.to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ));
        } else if raw_line.starts_with("* ") || raw_line.starts_with("- ") {
            let bullet = format!("  {} {}", "•", &raw_line[2..]);
            lines.push(Line::styled(bullet, Style::default().fg(Color::White)));
        } else if raw_line.starts_with("| ") {
            lines.push(Line::styled(raw_line.to_string(), Style::default().fg(Color::White)));
        } else {
            let spans = parse_inline_styles(raw_line);
            lines.push(Line::from(spans));
        }
    }

    // Handle unclosed code block
    if in_code_block && !code_buf.is_empty() {
        let highlighted = highlight_code_block(&code_buf, &code_lang);
        lines.extend(highlighted);
    }

    Text::from(lines)
}

/// Highlight a code block using syntect, falling back to plain green on failure.
fn highlight_code_block(code_lines: &[String], lang: &str) -> Vec<Line<'static>> {
    use ratatui::text::Span;
    use syntect::easy::HighlightLines;
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;

    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-eighties.dark"];

    // Find syntax by language token, fall back to plain text
    let syntax = ss
        .find_syntax_by_token(lang)
        .or_else(|| ss.find_syntax_by_extension(lang))
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut result = Vec::new();

    for line in code_lines {
        let line_with_nl = format!("{line}\n");
        match highlighter.highlight_line(&line_with_nl, &ss) {
            Ok(ranges) => {
                let spans: Vec<Span<'static>> = ranges
                    .into_iter()
                    .map(|(style, text)| {
                        let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                        Span::styled(text.trim_end_matches('\n').to_string(), Style::default().fg(fg))
                    })
                    .collect();
                result.push(Line::from(spans));
            }
            Err(_) => {
                result.push(Line::styled(line.to_string(), Style::default().fg(Color::Green)));
            }
        }
    }

    result
}

/// Parse inline **bold** and `code` markers into styled spans.
fn parse_inline_styles(line: &str) -> Vec<ratatui::text::Span<'static>> {
    use ratatui::text::Span;

    let mut spans = Vec::new();
    let mut remaining = line;

    while !remaining.is_empty() {
        // Check for **bold**
        if let Some(start) = remaining.find("**") {
            if start > 0 {
                spans.push(Span::raw(remaining[..start].to_string()));
            }
            let after_start = &remaining[start + 2..];
            if let Some(end) = after_start.find("**") {
                spans.push(Span::styled(
                    after_start[..end].to_string(),
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                remaining = &after_start[end + 2..];
                continue;
            } else {
                spans.push(Span::raw(remaining[start..].to_string()));
                break;
            }
        }
        // Check for `code`
        if let Some(start) = remaining.find('`') {
            if start > 0 {
                spans.push(Span::raw(remaining[..start].to_string()));
            }
            let after_start = &remaining[start + 1..];
            if let Some(end) = after_start.find('`') {
                spans.push(Span::styled(
                    after_start[..end].to_string(),
                    Style::default().fg(Color::Green),
                ));
                remaining = &after_start[end + 1..];
                continue;
            } else {
                spans.push(Span::raw(remaining[start..].to_string()));
                break;
            }
        }
        // No more markers
        spans.push(Span::raw(remaining.to_string()));
        break;
    }

    spans
}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////

impl TerminalConsole {
    pub fn new(commands: &UserCommands) -> Result<Self> {
        // Setup history
        let dirs = ProjectDirs::from(DIRS_QUALIFIER, DIRS_ORG, DIRS_APP).ok_or(Error::NotFound)?;
        let history_file = dirs.config_dir().join("history.txt");
        if !dirs.config_dir().exists() {
            fs::create_dir_all(dirs.config_dir())?;
        }
        let history = load_history(&history_file);

        // Collect command names for completion
        let command_names: Vec<String> = commands
            .list_commands()
            .iter()
            .flat_map(|c| {
                let mut names = vec![c.name.clone()];
                names.extend(c.alias.clone());
                names
            })
            .collect();

        // Generate banner
        let banner = generate_banner(format!("{PKG_NAME} {PKG_VERSION}"), "pagga");

        // Initialize terminal
        enable_raw_mode()?;
        let mut stderr = io::stderr();
        execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stderr);
        let terminal = Terminal::new(backend)?;

        let input = create_input();

        let mut console = Self {
            terminal,
            output_lines: Vec::new(),
            scroll_offset: 0,
            output_area_height: 0,
            spinner_active: false,
            spinner_frame: 0,
            input,
            banner,
            command_names,
            history,
            history_index: None,
            history_file,
        };

        console.render();

        Ok(console)
    }

    fn render(&mut self) {
        let banner = &self.banner;
        let output_lines = &self.output_lines;
        let scroll_offset = self.scroll_offset;
        let spinner_active = self.spinner_active;
        let spinner_frame = self.spinner_frame;
        let input = &self.input;
        let mut area_height: u16 = 0;

        let _ = self.terminal.draw(|frame| {
            let banner_height = banner.height() as u16;
            let spinner_height = if spinner_active { 1 } else { 0 };

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(banner_height),
                    Constraint::Min(1),
                    Constraint::Length(3),
                    Constraint::Length(spinner_height),
                ])
                .split(frame.area());

            area_height = chunks[1].height;

            // Banner
            let banner_widget = Paragraph::new(banner.clone()).style(Style::default().fg(Color::Cyan));
            frame.render_widget(banner_widget, chunks[0]);

            // Output area
            let mut combined = Text::default();
            for entry in output_lines {
                combined.extend(entry.clone());
                combined.push_line(Line::raw(""));
            }
            let output_widget = Paragraph::new(combined)
                .wrap(Wrap { trim: false })
                .scroll((scroll_offset, 0))
                .block(Block::default().borders(Borders::NONE));
            frame.render_widget(output_widget, chunks[1]);

            // Input area
            frame.render_widget(input, chunks[2]);

            // Spinner
            if spinner_active {
                let ch = SPINNER_CHARS[spinner_frame % SPINNER_CHARS.len()];
                let spinner_line = Line::styled(
                    format!(" {ch} Processing..."),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                );
                let spinner_widget = Paragraph::new(spinner_line);
                frame.render_widget(spinner_widget, chunks[3]);
            }
        });

        self.output_area_height = area_height;
    }

    fn content_height(&self) -> u16 {
        let mut total: u16 = 0;
        for entry in &self.output_lines {
            total = total.saturating_add(entry.height() as u16);
            total = total.saturating_add(1); // separator line
        }
        total
    }

    fn scroll_to_bottom(&mut self) {
        let content = self.content_height();
        let visible = self.output_area_height;
        if content > visible {
            self.scroll_offset = content.saturating_sub(visible);
        } else {
            self.scroll_offset = 0;
        }
    }

    fn scroll_up(&mut self, lines: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    fn scroll_down(&mut self, lines: u16) {
        let content = self.content_height();
        let visible = self.output_area_height;
        let max_scroll = content.saturating_sub(visible);
        self.scroll_offset = self.scroll_offset.saturating_add(lines).min(max_scroll);
    }

    fn add_to_history(&mut self, line: &str) {
        if self.history.last().map(|s| s.as_str()) != Some(line) {
            self.history.push(line.to_string());
        }
        self.history_index = None;
        save_history(&self.history_file, &self.history);
    }

    fn navigate_history_back(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let idx = match self.history_index {
            Some(i) => {
                if i > 0 {
                    i - 1
                } else {
                    0
                }
            }
            None => self.history.len() - 1,
        };
        self.history_index = Some(idx);
        self.set_input_text(&self.history[idx].clone());
    }

    fn navigate_history_forward(&mut self) {
        match self.history_index {
            Some(i) => {
                if i + 1 < self.history.len() {
                    self.history_index = Some(i + 1);
                    self.set_input_text(&self.history[i + 1].clone());
                } else {
                    self.history_index = None;
                    self.set_input_text("");
                }
            }
            None => {}
        }
    }

    fn set_input_text(&mut self, text: &str) {
        self.input = create_input();
        self.input.insert_str(text);
    }

    fn try_complete(&mut self) {
        let current = self.input.lines().join("");
        let current = current.trim().to_string();
        if current.is_empty() {
            return;
        }

        let matches: Vec<String> = self
            .command_names
            .iter()
            .filter(|name| name.starts_with(current.as_str()))
            .cloned()
            .collect();

        if matches.len() == 1 {
            self.set_input_text(&matches[0]);
        } else if matches.len() > 1 {
            let completions = matches.join("  ");
            let text = Text::styled(completions, Style::default().fg(Color::DarkGray));
            self.output_lines.push(text);
            self.scroll_to_bottom();
        }
    }

    fn handle_key_event(&mut self, key: event::KeyEvent) -> Option<InputAction> {
        match (key.code, key.modifiers) {
            // Submit on Enter
            (KeyCode::Enter, KeyModifiers::NONE) => {
                let text = self.input.lines().join("").trim().to_string();
                if !text.is_empty() {
                    self.add_to_history(&text);
                    self.input = create_input();
                    return Some(InputAction::Submit(text));
                }
                None
            }
            // Quit on Ctrl+D or Ctrl+C
            (KeyCode::Char('d'), KeyModifiers::CONTROL) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                Some(InputAction::Quit)
            }
            // History
            (KeyCode::Up, KeyModifiers::NONE) => {
                self.navigate_history_back();
                None
            }
            (KeyCode::Down, KeyModifiers::NONE) => {
                self.navigate_history_forward();
                None
            }
            // Scroll output
            (KeyCode::PageUp, _) => {
                self.scroll_up(10);
                None
            }
            (KeyCode::PageDown, _) => {
                self.scroll_down(10);
                None
            }
            // Tab completion
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.try_complete();
                None
            }
            // Pass everything else to textarea
            _ => {
                self.input.input(key);
                None
            }
        }
    }

    pub async fn read_input(&mut self) -> Result<String> {
        loop {
            self.render();

            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => match self.handle_key_event(key) {
                        Some(InputAction::Submit(text)) => return Ok(text),
                        Some(InputAction::Quit) => return Err(Error::EOF),
                        None => {}
                    },
                    Event::Mouse(mouse) => match mouse.kind {
                        event::MouseEventKind::ScrollUp => self.scroll_up(3),
                        event::MouseEventKind::ScrollDown => self.scroll_down(3),
                        _ => {}
                    },
                    Event::Resize(_, _) => {
                        // re-render on next iteration
                    }
                    _ => {}
                }
            }

            // Advance spinner animation during read_input loop
            if self.spinner_active {
                self.spinner_frame += 1;
            }
        }
    }

    pub fn display_error(&mut self, err: Error) -> Result<()> {
        match err {
            Error::LlmError { message } => self.display_string(message),
            _ => {
                let err_text = Text::styled(
                    format!("Error: {err}"),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                );
                self.output_lines.push(err_text);
                self.scroll_to_bottom();
                self.render();
                Ok(())
            }
        }
    }

    fn restore_terminal(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture);
        let _ = self.terminal.show_cursor();
    }
}

/// Polls crossterm events and resolves when Ctrl+C is pressed.
/// This is a standalone async function that doesn't need `&self`,
/// so it can run concurrently with handler code that borrows the console.
pub async fn wait_for_ctrl_c() {
    loop {
        // Non-blocking check for pending events
        if event::poll(Duration::ZERO).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                    return;
                }
            }
        }
        // Async sleep lets the handler future make progress
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

impl Drop for TerminalConsole {
    fn drop(&mut self) {
        self.restore_terminal();
    }
}

impl ConsoleDisplayTrait for TerminalConsole {
    fn start_spinner(&mut self) {
        self.spinner_active = true;
        self.spinner_frame = 0;
        self.render();
    }

    fn stop_spinner(&mut self) {
        self.spinner_active = false;
        self.render();
    }

    fn display<D>(&mut self, data: D) -> Result<()>
    where
        D: AsRef<AdoData>,
    {
        match data.as_ref() {
            AdoData::Empty => Ok(()),
            AdoData::Reset => {
                self.output_lines.clear();
                self.scroll_offset = 0;
                self.render();
                Ok(())
            }
            AdoData::Json(s) => {
                let json_md = format!("```json\n{s}\n```");
                self.display_string(json_md)
            }
            AdoData::String(s) => self.display_string(s),
            AdoData::Base64(_) => Ok(()),
            AdoData::SearchData(s) => {
                let md = s.to_markdown()?;
                self.display_string(md)
            }
            AdoData::UsageString(s) => self.display_string(s),
            AdoData::Shell(s) => {
                let md = s.to_markdown()?;
                self.display_string(md)
            }
            AdoData::Status(s) => {
                let md = s.to_markdown()?;
                self.display_string(md)
            }
            AdoData::LlmUsage(u) => {
                let md = u.to_markdown()?;
                self.display_string(md)
            }
            AdoData::Bytes(_) => Ok(()),
        }
    }

    fn display_string<S>(&mut self, value: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        let text = render_markdown(value.as_ref());
        self.output_lines.push(text);
        self.scroll_to_bottom();
        self.render();
        Ok(())
    }
}
