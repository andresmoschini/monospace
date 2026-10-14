//! What one frame shows, as a pure function of the model.
//!
//! # Design notes
//!
//! **`view` takes `&Model` and returns an `impl Widget`, with no `Frame` and no `Backend`.** That is
//! the whole of what makes the drawing testable: the function is pure, so a test renders it into a
//! [`TestBackend`](ratatui::backend::TestBackend) and reads the buffer back, with no terminal involved. `draw` in `main` is the
//! thin adapter that hands the widget to the `Frame`; it holds no logic of its own.
//!
//! **What the model means for the picture stays here.** The test asserts on the widget the model
//! produces rather than on a terminal, so when the model grows visible state, the widget tree grows
//! here and the root stays the same.

use ratatui::widgets::{Block, Widget};

use crate::model::Model;

/// Builds the widget tree for one frame from the model's current state.
pub(crate) fn view(model: &Model) -> impl Widget {
    let state = if model.should_quit() {
        "done"
    } else {
        "running"
    };

    Block::bordered().title(state)
}

#[cfg(test)]
mod tests {
    use super::{Model, view};
    use crate::message::Message;
    use ratatui::backend::TestBackend;
    use ratatui::{Terminal, buffer::Buffer};

    fn render(model: &Model) -> Buffer {
        let backend = TestBackend::new(10, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| frame.render_widget(view(model), frame.area()))
            .unwrap();

        terminal.backend().buffer().clone()
    }

    fn top_row(buffer: &Buffer) -> String {
        (buffer.area.left()..buffer.area.right())
            .map(|x| buffer[(x, buffer.area.top())].symbol().to_string())
            .collect()
    }

    #[test]
    fn the_running_view_draws_a_titled_block() {
        let buffer = render(&Model::new());

        assert_eq!(buffer[(0, 0)].symbol(), "┌");
        assert_eq!(buffer[(9, 0)].symbol(), "┐");
        assert_eq!(buffer[(0, 2)].symbol(), "└");
        assert_eq!(buffer[(9, 2)].symbol(), "┘");
        assert!(top_row(&buffer).contains("running"));
    }

    #[test]
    fn the_done_view_says_done() {
        let model = Model::new().update(Message::Quit);
        let buffer = render(&model);

        assert!(top_row(&buffer).contains("done"));
    }
}
