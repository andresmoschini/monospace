//! The interactive application: the loop, and the terminal it runs on.

mod app;
mod message;
mod model;
mod terminal;
mod view;

use std::{io::stdout, process::ExitCode};

use ratatui::{Terminal, backend::CrosstermBackend};

use crate::app::App;
use crate::terminal::{Crossterm, TerminalGuard};

fn main() -> ExitCode {
    let mut terminal = match Terminal::new(CrosstermBackend::new(stdout())) {
        Ok(terminal) => terminal,
        Err(error) => {
            eprintln!("monospace: the terminal could not be opened: {error}");
            return ExitCode::FAILURE;
        }
    };

    // Bound here and dropped on every way out of `main`, including one that arrives as a panic
    // unwinding through `run` below.
    let guard = match TerminalGuard::take(Box::new(Crossterm)) {
        Ok(guard) => guard,
        Err(error) => {
            eprintln!("monospace: the terminal could not be taken: {error}");
            return ExitCode::FAILURE;
        }
    };

    let mut app = App::new();

    let outcome = app.run(&mut terminal);

    drop(guard);

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("monospace: {error}");
            ExitCode::FAILURE
        }
    }
}
