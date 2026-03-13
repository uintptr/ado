use std::{
    fs,
    io::{self, stdout},
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    ExecutableCommand, cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    terminal,
};
use log::error;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
};
use tui_input::backend::crossterm::EventHandler;

use crate::{
    banner::render_banner,
    commands::UserCommands,
    input::{InputState, MAX_SUGGESTIONS, accept_suggestion, history_next, history_prev, update_suggestions},
    terminal::{PKG_NAME, PKG_VERSION, TuiConsole},
};

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub enum TuiEvent {
    PlainLine(String),
    StyledLines(Vec<Line<'static>>),
    Separator,
    ThinkingStart,
    ThinkingStop,
    Done,
    Clear,
}

#[derive(PartialEq)]
enum AppMode {
    Input,
    Thinking,
}

struct TuiApp {
    output: Vec<Line<'static>>,
    scroll_offset: usize,
    auto_scroll: bool,
    input: InputState,
    mode: AppMode,
    spinner_frame: usize,
    event_rx: Receiver<TuiEvent>,
    input_tx: Sender<String>,
    history: Vec<String>,
    commands: Vec<String>,
}

impl TuiApp {
    fn new(
        event_rx: Receiver<TuiEvent>,
        input_tx: Sender<String>,
        history: Vec<String>,
        commands: Vec<String>,
    ) -> Self {
        Self {
            output: Vec::new(),
            scroll_offset: 0,
            auto_scroll: true,
            input: InputState::new(),
            mode: AppMode::Input,
            spinner_frame: 0,
            event_rx,
            input_tx,
            history,
            commands,
        }
    }

    fn push_line(&mut self, line: Line<'static>) {
        self.output.push(line);
    }

    fn push_separator(&mut self, width: usize) {
        let w = width.min(500);
        self.output.push(Line::from(Span::styled(
            "─".repeat(w),
            Style::default().fg(Color::DarkGray),
        )));
    }

    fn drain_events(&mut self, output_width: usize) {
        loop {
            match self.event_rx.try_recv() {
                Ok(TuiEvent::PlainLine(s)) => {
                    self.push_line(Line::from(s));
                }
                Ok(TuiEvent::StyledLines(lines)) => {
                    self.output.extend(lines);
                }
                Ok(TuiEvent::Separator) => self.push_separator(output_width),
                Ok(TuiEvent::ThinkingStart) => {
                    self.mode = AppMode::Thinking;
                }
                Ok(TuiEvent::ThinkingStop) => {}
                Ok(TuiEvent::Done) => {
                    self.mode = AppMode::Input;
                }
                Ok(TuiEvent::Clear) => {
                    self.output.clear();
                    self.scroll_offset = 0;
                    self.auto_scroll = true;
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
    }

    /// Number of visual rows a single output line occupies when wrapped to `width` columns.
    fn line_visual_rows(line: &Line, width: usize) -> usize {
        if width == 0 {
            return 1;
        }
        let w = line.width();
        if w == 0 { 1 } else { w.div_ceil(width) }
    }

    /// Maximum number of lines to skip so the remaining lines still fill the viewport.
    /// Returns the line index of the first visible line when auto-scrolled to the bottom.
    fn max_scroll_lines(&self, output_width: usize, viewport_height: usize) -> usize {
        // Walk backwards accumulating visual rows until we've filled the viewport.
        let mut rows = 0usize;
        for (i, line) in self.output.iter().enumerate().rev() {
            rows = rows.saturating_add(Self::line_visual_rows(line, output_width));
            if rows >= viewport_height {
                return i;
            }
        }
        0
    }

    /// Total visual rows across all output lines (for scrollbar sizing).
    fn total_visual_rows(&self, output_width: usize) -> usize {
        self.output.iter().map(|l| Self::line_visual_rows(l, output_width)).sum()
    }

    /// Visual row offset corresponding to `scroll_offset` lines skipped.
    fn visual_scroll_offset(&self, output_width: usize) -> usize {
        self.output
            .iter()
            .take(self.scroll_offset)
            .map(|l| Self::line_visual_rows(l, output_width))
            .sum()
    }

    fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
        self.auto_scroll = false;
    }

    fn scroll_down(&mut self, amount: usize, output_width: usize, viewport_height: usize) {
        let max_scroll = self.max_scroll_lines(output_width, viewport_height);
        self.scroll_offset = self.scroll_offset.saturating_add(amount).min(max_scroll);
        if self.scroll_offset >= max_scroll {
            self.auto_scroll = true;
        }
    }

    fn clamp_scroll(&mut self, output_width: usize, viewport_height: usize) {
        let max_scroll = self.max_scroll_lines(output_width, viewport_height);
        if self.auto_scroll {
            self.scroll_offset = max_scroll;
        } else {
            self.scroll_offset = self.scroll_offset.min(max_scroll);
        }
    }

    fn submit_input(&mut self) {
        if self.mode != AppMode::Input {
            return;
        }
        let line = self.input.value().trim().to_string();
        if line.is_empty() {
            return;
        }
        self.history.push(line.clone());
        self.input.input = tui_input::Input::default();
        self.input.history_index = None;

        // Echo the user's input in the output pane
        self.push_line(Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Green)),
            Span::raw(line.clone()),
        ]));
        self.auto_scroll = true;

        self.mode = AppMode::Thinking;
        let _ = self.input_tx.send(line);
    }
}

fn render_output_pane(frame: &mut ratatui::Frame, app: &TuiApp, output_area: Rect) {
    let visible_lines: Vec<Line> = app.output.iter().skip(app.scroll_offset).cloned().collect();

    frame.render_widget(Paragraph::new(visible_lines).wrap(Wrap { trim: false }), output_area);

    let output_height = output_area.height as usize;
    let output_width = output_area.width as usize;
    let total_visual = app.total_visual_rows(output_width);
    if total_visual > output_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);
        let visual_pos = app.visual_scroll_offset(output_width);
        let mut state = ScrollbarState::new(total_visual.saturating_sub(output_height)).position(visual_pos);
        frame.render_stateful_widget(scrollbar, output_area, &mut state);
    }
}

fn render_popup(frame: &mut ratatui::Frame, app: &TuiApp, input_area: Rect, popup_height: u16) {
    if !app.input.show_popup || app.input.suggestions.is_empty() || popup_height == 0 {
        return;
    }
    let popup_area = Rect {
        x: input_area.x,
        y: input_area.y.saturating_sub(popup_height),
        width: input_area.width,
        height: popup_height,
    };
    let items: Vec<ListItem> = app
        .input
        .suggestions
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == app.input.suggestion_index {
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

fn render_input_bar(frame: &mut ratatui::Frame, app: &TuiApp, input_area: Rect) {
    let (prompt_str, prompt_style) = match app.mode {
        AppMode::Thinking => {
            let frame_char = app
                .spinner_frame
                .checked_rem(SPINNER_FRAMES.len())
                .and_then(|i| SPINNER_FRAMES.get(i))
                .copied()
                .unwrap_or("⠋");
            (format!("{frame_char} "), Style::default().fg(Color::Yellow))
        }
        AppMode::Input => ("> ".to_string(), Style::default().fg(Color::Green)),
    };
    let input_value = if app.mode == AppMode::Input {
        app.input.value().to_string()
    } else {
        String::new()
    };
    let input_line = Line::from(vec![
        Span::styled(prompt_str.clone(), prompt_style),
        Span::raw(input_value),
    ]);
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(
        Paragraph::new(input_line).block(input_block).wrap(Wrap { trim: false }),
        input_area,
    );

    if app.mode == AppMode::Input {
        // inner width = area width minus 2 border chars
        let inner_width = input_area.width.saturating_sub(2) as usize;
        let prompt_len = prompt_str.len();
        let cursor_offset = prompt_len.saturating_add(app.input.cursor());
        let (cursor_row, cursor_col) = if inner_width > 0 {
            let row = cursor_offset.checked_div(inner_width).unwrap_or(0);
            let col = cursor_offset.checked_rem(inner_width).unwrap_or(0);
            (row, col)
        } else {
            (0, cursor_offset)
        };
        let cursor_x = input_area
            .x
            .saturating_add(1)
            .saturating_add(u16::try_from(cursor_col).unwrap_or(u16::MAX));
        let cursor_y = input_area
            .y
            .saturating_add(1)
            .saturating_add(u16::try_from(cursor_row).unwrap_or(u16::MAX));
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn render(frame: &mut ratatui::Frame, app: &TuiApp) {
    let popup_height: u16 = if app.input.show_popup && !app.input.suggestions.is_empty() {
        u16::try_from(app.input.suggestions.len().min(MAX_SUGGESTIONS))
            .unwrap_or(u16::MAX)
            .saturating_add(2)
    } else {
        0
    };

    // Calculate how many lines the input needs when wrapped.
    // inner_width = frame width - 2 borders; prompt is "> " (2 chars)
    let frame_width = frame.area().width;
    let inner_width = frame_width.saturating_sub(2) as usize;
    let input_text_len = if app.mode == AppMode::Input {
        app.input.value().len().saturating_add(2) // 2 for "> "
    } else {
        2
    };
    let input_lines = if inner_width > 0 {
        u16::try_from(input_text_len.div_ceil(inner_width))
            .unwrap_or(1)
            .max(1)
    } else {
        1
    };
    // +2 for top/bottom border
    let input_height = input_lines.saturating_add(2);

    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(input_height)]).split(frame.area());
    let Some(output_area) = chunks.first().copied() else {
        return;
    };
    let Some(input_area) = chunks.get(1).copied() else {
        return;
    };

    render_output_pane(frame, app, output_area);
    render_popup(frame, app, input_area, popup_height);
    render_input_bar(frame, app, input_area);
}

fn handle_input_key(app: &mut TuiApp, key: event::KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            if app.input.show_popup && !app.input.suggestions.is_empty() {
                accept_suggestion(&mut app.input);
                update_suggestions(&mut app.input, &app.commands);
            } else {
                app.submit_input();
            }
        }
        KeyCode::Tab => {
            if app.input.show_popup {
                accept_suggestion(&mut app.input);
                update_suggestions(&mut app.input, &app.commands);
            }
        }
        KeyCode::Up => {
            if app.input.show_popup {
                if app.input.suggestion_index > 0 {
                    app.input.suggestion_index = app.input.suggestion_index.saturating_sub(1);
                }
            } else {
                history_prev(&mut app.input, &app.history);
            }
        }
        KeyCode::Down => {
            if app.input.show_popup {
                if app.input.suggestion_index.saturating_add(1) < app.input.suggestions.len() {
                    app.input.suggestion_index = app.input.suggestion_index.saturating_add(1);
                }
            } else {
                history_next(&mut app.input, &app.history);
            }
        }
        KeyCode::Esc => {
            if app.input.show_popup {
                app.input.show_popup = false;
                app.input.suggestions.clear();
            } else {
                app.input.input = tui_input::Input::default();
            }
        }
        _ => {
            app.input.input.handle_event(&Event::Key(key));
            update_suggestions(&mut app.input, &app.commands);
        }
    }
}

fn run_event_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut TuiApp) -> Result<()> {
    let tick = Duration::from_millis(50);

    loop {
        let size = terminal.size()?;
        let output_height = size.height.saturating_sub(3) as usize;
        let output_width = size.width as usize;

        app.drain_events(output_width);
        app.clamp_scroll(output_width, output_height);

        if app.mode == AppMode::Thinking {
            app.spinner_frame = app.spinner_frame.wrapping_add(1);
        }

        terminal.draw(|f| render(f, app))?;

        if !event::poll(tick)? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                continue;
            }
            match key.code {
                KeyCode::Char('c' | 'd') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(());
                }
                KeyCode::PageUp => {
                    let half = output_height / 2;
                    app.scroll_up(half.max(1));
                }
                KeyCode::PageDown => {
                    let half = output_height / 2;
                    app.scroll_down(half.max(1), output_width, output_height);
                }
                _ if app.mode == AppMode::Input => {
                    handle_input_key(app, key);
                }
                _ => {}
            }
        }
    }
}

fn processing_loop(
    mut commands: UserCommands,
    console: &TuiConsole,
    input_rx: &Receiver<String>,
    event_tx: &Sender<TuiEvent>,
) {
    while let Ok(input) = input_rx.recv() {
        if let Err(e) = commands.handler(&input, console) {
            error!("handler error: {e}");
            let _ = event_tx.send(TuiEvent::PlainLine(format!("Error: {e}")));
        }
        let _ = event_tx.send(TuiEvent::Done);
    }
}

pub fn load_history(history_file: &PathBuf) -> Vec<String> {
    fs::read_to_string(history_file)
        .unwrap_or_default()
        .lines()
        .map(String::from)
        .collect()
}

fn save_history(path: &PathBuf, history: &[String]) {
    let start = history.len().saturating_sub(1000);
    let content = history.get(start..).unwrap_or_default().join("\n");
    let _ = fs::write(path, content);
}

pub fn run(
    commands: UserCommands,
    history: Vec<String>,
    history_file: &PathBuf,
    command_names: Vec<String>,
) -> Result<()> {
    let (input_tx, input_rx) = mpsc::channel::<String>();
    let (event_tx, event_rx) = mpsc::channel::<TuiEvent>();

    let event_tx_for_thread = event_tx.clone();
    let process_thread = thread::spawn(move || {
        let console = TuiConsole::new(event_tx_for_thread.clone());
        processing_loop(commands, &console, &input_rx, &event_tx_for_thread);
    });

    // Enter alternate screen
    terminal::enable_raw_mode()?;
    let mut out = stdout();
    out.execute(terminal::EnterAlternateScreen)?;
    out.execute(terminal::Clear(terminal::ClearType::All))?;
    out.execute(cursor::Hide)?;

    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let mut app = TuiApp::new(event_rx, input_tx, history, command_names);

    // Seed output buffer with the figlet banner
    if let Ok(banner) = render_banner(format!("{PKG_NAME} {PKG_VERSION}"), "pagga") {
        for line in banner.lines() {
            app.push_line(Line::from(line.to_string()));
        }
    }

    let result = run_event_loop(&mut terminal, &mut app);

    // Restore terminal
    terminal::disable_raw_mode()?;
    let mut out = io::stdout();
    out.execute(terminal::LeaveAlternateScreen)?;
    out.execute(cursor::Show)?;

    // Save history
    save_history(history_file, &app.history);

    // Signal processing thread to stop and wait
    drop(app.input_tx);
    let _ = process_thread.join();

    result
}
