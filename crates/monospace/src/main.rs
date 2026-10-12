//! The interactive application: the loop, and the terminal it runs on.

use std::{io::stdout, process::ExitCode};

use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> ExitCode {
    match Terminal::new(CrosstermBackend::new(stdout())) {
        Ok(terminal) => terminal,
        Err(error) => {
            eprintln!("monospace: the terminal could not be opened: {error}");
            return ExitCode::FAILURE;
        }
    };

    eprintln!("monospace: NOT IMPLEMENTED");
    ExitCode::FAILURE
}
