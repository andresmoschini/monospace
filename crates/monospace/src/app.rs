//! The application: the model it holds, the loop that runs it, and the frame it draws.
//!
//! # Design notes
//!
//! **`App` owns the `Model` and is the only thing that touches it.** The loop lives in `run`, and
//! `update` and `draw` are methods that read or replace `self.model` — so the model is never passed
//! by parameter through the loop, and there is one owner rather than a value threaded through every
//! call. **What breaks if it changes:** a `Model` that stops being `Clone` forces `update` to
//! mutate in place, and the `model = ...` rebinds in `run` and `update` become assignments.

use std::io;

use crossterm::event;
use ratatui::Frame;
use ratatui::Terminal;

use crate::message::Message;
use crate::model::Model;
use crate::view::view;

/// The running application: the state it holds and the loop that drives it.
pub(crate) struct App {
    model: Model,
}

impl App {
    /// The application the loop starts from.
    pub(crate) fn new() -> Self {
        Self {
            model: Model::new(),
        }
    }

    /// Runs the loop until the model says to stop, giving the terminal back on the way out.
    pub(crate) fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> io::Result<()>
    where
        B::Error: IntoTerminalError,
    {
        while !self.model.should_quit() {
            if self.model.should_redraw() {
                terminal
                    .draw(|frame| self.draw(frame))
                    .map_err(B::Error::into_terminal_error)?;
                self.model = self.model.mark_drawn();
            }

            let event = event::read()?;
            let message = Message::parse(&event);

            if let Some(message) = message {
                self.update(message);
            }
        }
        Ok(())
    }

    /// Applies one message, replacing the model it leaves behind.
    fn update(&mut self, message: Message) {
        self.model = self.model.update(message);
    }

    /// Draws the current model into the frame.
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(view(&self.model), frame.area());
    }
}

pub(crate) trait IntoTerminalError {
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
