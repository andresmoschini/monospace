//! The interactive application: the loop, and the terminal it runs on.

use std::io;
use std::{io::stdout, process::ExitCode};

use crossterm::event::{self, Event};
use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> ExitCode {
    match Terminal::new(CrosstermBackend::new(stdout())) {
        Ok(terminal) => terminal,
        Err(error) => {
            eprintln!("monospace: the terminal could not be opened: {error}");
            return ExitCode::FAILURE;
        }
    };

    let outcome = run();

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("monospace: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<()> {
    loop {
        let event = next()?;

        if let Event::Key(key) = event
            && key.kind == event::KeyEventKind::Press
        {
            match key.code {
                event::KeyCode::Char('o') => return Ok(()),
                event::KeyCode::Char('e') => return Err(io::Error::other("e pressed")),
                _ => {}
            }
        }
    }
}

fn next() -> io::Result<Event> {
    event::read()
}
