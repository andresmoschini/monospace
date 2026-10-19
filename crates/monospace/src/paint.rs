//! What one frame shows, as a pure function of the area, the state and the configuration.
//!
//! `draw` takes a `Buffer` and returns nothing, which is what makes it testable: a test reads the
//! cells back with no terminal and no `Frame` involved. The widget tree, the `impl Widget` and the
//! `TestBackend` went away with the loop — the render callback already owns the frame.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Widget};

use crate::context::AppGlobal;
use crate::state::AppState;

/// Draws one frame into the buffer.
pub(crate) fn draw(area: Rect, buf: &mut Buffer, state: &AppState, global: &AppGlobal) {
    let title = if global.cfg.counter {
        format!("{} {}", global.cfg.title, state.counter)
    } else {
        global.cfg.title.clone()
    };

    Block::bordered().title(title).render(area, buf);
}

#[cfg(test)]
mod tests {
    use super::draw;
    use crate::config::Config;
    use crate::context::AppGlobal;
    use crate::state::AppState;
    use monospace_diagram::Diagram;
    use ratatui::{buffer::Buffer, layout::Rect};

    fn render(width: u16, height: u16, cfg: Config) -> Buffer {
        let state = AppState::new(Diagram::new());
        let global = AppGlobal::new(cfg);
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
        draw(Rect::new(0, 0, width, height), &mut buffer, &state, &global);
        buffer
    }

    fn top_row(buffer: &Buffer) -> String {
        (buffer.area.left()..buffer.area.right())
            .map(|x| buffer[(x, buffer.area.top())].symbol().to_string())
            .collect()
    }

    #[test]
    fn the_view_draws_a_titled_block() {
        let buffer = render(20, 3, Config::default());

        assert_eq!(buffer[(0, 0)].symbol(), "┌");
        assert_eq!(buffer[(19, 0)].symbol(), "┐");
        assert_eq!(buffer[(0, 2)].symbol(), "└");
        assert_eq!(buffer[(19, 2)].symbol(), "┘");
        assert!(top_row(&buffer).contains("monospace 0"));
    }

    #[test]
    fn the_configuration_reaches_the_frame() {
        let cfg = Config {
            title: "probe".to_string(),
            counter: false,
        };
        let buffer = render(12, 3, cfg);
        let top = top_row(&buffer);

        assert!(
            top.contains("probe"),
            "the configured title is absent: {top:?}"
        );
        assert!(
            !top.contains('0'),
            "the counter is drawn although it is off: {top:?}"
        );
    }
}
