//! What one frame shows, as a pure function of the area.
//!
//! # Design notes
//!
//! **`draw` takes a `Rect` and a `Buffer`, and returns nothing.** That is the whole of what makes
//! the drawing testable: the function is pure, so a test hands it an empty
//! [`Buffer`] the size of the screen and reads the cells back, with no
//! terminal and no `Frame` involved. What used to sit between them — the widget tree, the
//! `impl Widget`, the `TestBackend` — is gone with the loop: the framework's render callback
//! already owns the frame, so a widget to hand to it would be a ceremony with nothing to do.
//!
//! **What the frame shows is here and nowhere else.** The title is the application's name, because
//! the state it used to report — running, and the frame after the way out — belonged to the loop
//! and is `Control::Quit` now. The second was never drawn: the loop ended before the frame that
//! would have shown it.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Widget};

/// Draws one frame into the buffer.
pub(crate) fn draw(area: Rect, buf: &mut Buffer) {
    Block::bordered().title("mono").render(area, buf);
}

#[cfg(test)]
mod tests {
    use super::draw;
    use ratatui::{buffer::Buffer, layout::Rect};

    fn render(width: u16, height: u16) -> Buffer {
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
        draw(Rect::new(0, 0, width, height), &mut buffer);
        buffer
    }

    fn top_row(buffer: &Buffer) -> String {
        (buffer.area.left()..buffer.area.right())
            .map(|x| buffer[(x, buffer.area.top())].symbol().to_string())
            .collect()
    }

    #[test]
    fn the_view_draws_a_titled_block() {
        let buffer = render(10, 3);

        assert_eq!(buffer[(0, 0)].symbol(), "┌");
        assert_eq!(buffer[(9, 0)].symbol(), "┐");
        assert_eq!(buffer[(0, 2)].symbol(), "└");
        assert_eq!(buffer[(9, 2)].symbol(), "┘");
        assert!(top_row(&buffer).contains("mono"));
    }
}
