use std::{
    io::{self, Write},
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::{Result, bail};
use colored::Colorize;
use log::{error, info};

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

fn spinner(rx: Receiver<SpinMessage>) {
    let mut stdout = io::stdout();

    loop {
        // Block until we get a message
        match rx.recv() {
            Err(_) => return, // sender dropped
            Ok(SpinMessage::Stop) => continue,
            Ok(SpinMessage::Start) => {}
            Ok(SpinMessage::Quit) => return,
        }

        // Keep spinning until Stop arrives
        for frame in FRAMES.iter().cycle() {
            print!("\r{}", frame.green());

            stdout.flush().unwrap();

            thread::sleep(Duration::from_millis(100));

            match rx.try_recv() {
                Ok(SpinMessage::Stop) => {
                    print!("\r            ");
                    stdout.flush().unwrap();
                    break;
                }
                Ok(SpinMessage::Quit) => return,
                Ok(SpinMessage::Start) | Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => return,
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
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        let thread_handle = Some(thread::spawn(move || spinner(rx)));

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
    }

    pub fn quit(&mut self) -> Result<()> {
        info!("Joining Spinner");

        if let Some(handle) = self.thread_handle.take() {
            self.tx.send(SpinMessage::Quit)?;
            match handle.join() {
                Ok(_) => Ok(()),
                Err(_) => bail!("Unable to join spinner thread"),
            }
        } else {
            Ok(())
        }
    }
}
