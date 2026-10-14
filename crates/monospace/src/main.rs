//! The interactive application: the loop, and the terminal it runs on.

mod message;
mod model;
mod terminal;

use std::io;
use std::{io::stdout, process::ExitCode};

use crossterm::event;
use ratatui::Frame;
use ratatui::widgets::Block;
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::message::Message;
use crate::model::Model;
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

    let outcome = run(&mut terminal);

    drop(guard);

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("monospace: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()>
where
    B::Error: IntoTerminalError,
{
    let mut model = Model::new();

    while !model.should_quit() {
        if model.should_redraw() {
            terminal
                .draw(|frame| draw(frame, &model))
                .map_err(B::Error::into_terminal_error)?;
            model = model.mark_drawn();
        }

        let event = event::read()?;
        eprintln!("{event:?}");

        let message = Message::parse(&event);
        eprintln!("{message:?}");

        if let Some(message) = message {
            model = model.update(message);
        }
    }
    Ok(())
}

fn draw(frame: &mut Frame, model: &Model) {
    eprintln!("{model:?}");
    frame.render_widget(Block::bordered(), frame.area());
}

trait IntoTerminalError {
    fn into_terminal_error(self) -> io::Error;
}

impl IntoTerminalError for io::Error {
    fn into_terminal_error(self) -> io::Error {
        self
    }
}

impl IntoTerminalError for std::convert::Infallible {
    fn into_terminal_error(self) -> io::Error {
        match self {}
    }
}
