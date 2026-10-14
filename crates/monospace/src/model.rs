//! The state the loop holds between one draw and the next, and the messages that change it.
//!
//! # Design notes
//!
//! **`update` takes `&self` and returns the model it leaves behind**, the Elm shape, and the
//! loop rebinds: `model = model.update(message)`. It is a deliberate choice over `&mut self`:
//! the model is rebuilt whole on each message, which is cheap while the state is small and keeps
//! the loop free of mutation. **What breaks if it changes:** an `update` that mutates in place
//! turns the rebind into a call, and `should_quit` becomes the loop's only reader.
//!
//! **`redraw` is the hook for incremental rendering**: the loop draws only when `should_quit` is
//! false and `should_redraw` is true, and calls `mark_drawn` after a frame, which clears the flag.
//! A message that changes something visible sets it true again, so the next frame draws; a message
//! that changes nothing leaves it false and the frame is skipped. Today no message sets it true, so
//! only the first frame draws.
//!
//! **The loop asks the model whether to run**, through `running_state`, rather than `update`
//! returning a verdict: what a message means stays in the model, and the loop matches nothing.
//!
//! **Today the model holds only whether a message asked the loop to end**, the honest minimum:
//! one message exists, and the one thing the loop must learn from it is when to stop.

use crate::message::Message;

/// Whether the loop should keep drawing or give the terminal back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RunningState {
    /// The application runs; the loop draws and waits for the next event.
    Running,
    /// A message asked the application to end; the loop gives the terminal back.
    Done,
}

/// The state the application holds between one draw and the next.
#[derive(Debug, Clone)]
pub(crate) struct Model {
    /// Whether the loop should keep running.
    running_state: RunningState,
    /// Whether the next frame is worth drawing; the loop skips `draw` when this is false.
    redraw: bool,
}

impl Default for Model {
    /// The model the application starts from, which is [`new`](Self::new).
    fn default() -> Self {
        Self::new()
    }
}

impl Model {
    /// The model the application starts from.
    pub(crate) fn new() -> Self {
        Model {
            running_state: RunningState::Running,
            redraw: true,
        }
    }

    /// Applies one message, returning the model it leaves behind.
    pub(crate) fn update(&self, message: Message) -> Self {
        match message {
            Message::Quit => Self {
                running_state: RunningState::Done,
                redraw: self.redraw,
            },
        }
    }

    /// Whether the loop has been told to end.
    pub(crate) fn should_quit(&self) -> bool {
        self.running_state == RunningState::Done
    }

    /// Whether the loop should draw this frame.
    pub(crate) fn should_redraw(&self) -> bool {
        self.redraw
    }

    /// Marks the model as drawn, returning a copy that will not ask for a redraw until a message
    /// sets the flag again.
    pub(crate) fn mark_drawn(&self) -> Self {
        Self {
            redraw: false,
            ..self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Model;
    use crate::message::Message;

    #[test]
    fn a_new_model_is_running_and_redraws() {
        let model = Model::new();

        assert!(!model.should_quit());
        assert!(model.should_redraw());
    }

    #[test]
    fn a_quit_message_tells_the_loop_to_end() {
        let model = Model::new();

        let model = model.update(Message::Quit);

        assert!(model.should_quit());
    }

    #[test]
    fn update_carries_the_redraw_flag_forward() {
        let model = Model::new();

        let model = model.update(Message::Quit);

        assert!(model.should_redraw());
    }

    #[test]
    fn marking_the_model_drawn_clears_the_redraw_flag() {
        let model = Model::new();

        let model = model.mark_drawn();

        assert!(!model.should_redraw());
    }

    #[test]
    fn marking_the_model_drawn_keeps_the_running_state() {
        let model = Model::new().update(Message::Quit);

        let model = model.mark_drawn();

        assert!(model.should_quit());
    }
}
