use std::{
    io,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread::{self, JoinHandle, sleep},
    time::Duration,
};

use anyhow::{Result, bail};
use crossterm::{cursor, execute, style, terminal};
use log::{error, info};

use crate::ui;

const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

enum SpinMessage {
    Start,
    Stop,
    Quit,
}

pub struct AdoSpinner {
    tx: Sender<SpinMessage>,
    thread_handle: Option<JoinHandle<()>>,
}

fn draw_box(stdout: &mut io::Stdout, frame: &str) {
    let w = ui::terminal_width();
    let inner = w.saturating_sub(2);
    let dashes = "─".repeat(inner);

    // Top border with spinner title embedded
    let title = format!(" {frame} Thinking... ");
    let title_len = title.len();
    let fill = inner.saturating_sub(title_len).saturating_sub(1); // -1 for leading ─
    let top = format!("┌─{title}{}┐", "─".repeat(fill));

    let content = format!("│{}│", " ".repeat(inner));
    let bottom = format!("└{dashes}┘");

    let _ = execute!(
        stdout,
        cursor::MoveToColumn(0),
        terminal::Clear(terminal::ClearType::FromCursorDown),
        style::SetForegroundColor(style::Color::DarkGrey),
        style::Print(format!("{top}\n{content}\n{bottom}")),
        style::ResetColor,
        // Move back to top of box for next frame redraw
        cursor::MoveUp(2),
        cursor::MoveToColumn(0),
    );
}

fn clear_box(stdout: &mut io::Stdout) {
    let _ = execute!(
        stdout,
        cursor::MoveToColumn(0),
        terminal::Clear(terminal::ClearType::FromCursorDown),
    );
}

fn spinner(rx: &Receiver<SpinMessage>) {
    let mut stdout = io::stdout();

    loop {
        // Block until we get a message
        match rx.recv() {
            Ok(SpinMessage::Stop) => continue,
            Ok(SpinMessage::Start) => {}
            Err(_) | Ok(SpinMessage::Quit) => return,
        }

        // Keep spinning until Stop arrives
        for frame in FRAMES.iter().cycle() {
            draw_box(&mut stdout, frame);

            thread::sleep(Duration::from_millis(80));

            match rx.try_recv() {
                Ok(SpinMessage::Stop) => {
                    clear_box(&mut stdout);
                    break;
                }
                Ok(SpinMessage::Quit) | Err(TryRecvError::Disconnected) => return,
                Ok(SpinMessage::Start) | Err(TryRecvError::Empty) => {}
            }
        }
    }
}

impl Default for AdoSpinner {
    fn default() -> Self {
        Self::new()
    }
}

impl AdoSpinner {
    #[must_use]
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        let thread_handle = Some(thread::spawn(move || spinner(&rx)));

        Self { tx, thread_handle }
    }

    pub fn start(&self) {
        if let Err(e) = self.tx.send(SpinMessage::Start) {
            error!("unable to message spinner ({e})");
        }
    }
    pub fn stop(&self) {
        if let Err(e) = self.tx.send(SpinMessage::Stop) {
            error!("unable to message spinner ({e})");
        }
        sleep(Duration::from_millis(100));
    }

    pub fn quit(&mut self) -> Result<()> {
        info!("Joining Spinner");

        if let Some(handle) = self.thread_handle.take() {
            self.tx.send(SpinMessage::Quit)?;
            match handle.join() {
                Ok(()) => Ok(()),
                Err(_) => bail!("Unable to join spinner thread"),
            }
        } else {
            Ok(())
        }
    }
}
