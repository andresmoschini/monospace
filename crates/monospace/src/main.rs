//! The interactive application: the loop, and the terminal it runs on.

use std::io;
use std::{io::stdout, process::ExitCode};

use crossterm::event::{self, Event};
use ratatui::Frame;
use ratatui::widgets::Block;
use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> ExitCode {
    let mut terminal = match Terminal::new(CrosstermBackend::new(stdout())) {
        Ok(terminal) => terminal,
        Err(error) => {
            eprintln!("monospace: the terminal could not be opened: {error}");
            return ExitCode::FAILURE;
        }
    };

    let outcome = run(&mut terminal);

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
    loop {
        terminal.draw(draw).map_err(B::Error::into_terminal_error)?;

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

fn draw(frame: &mut Frame) {
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
