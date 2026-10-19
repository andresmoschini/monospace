//! What one frame shows, as a pure function of the area, the state and the configuration.
//!
//! **The order of the three draws is the picture.** The bar, then what is left of the screen, then
//! the drop-down: a widget drawn later covers the one before it, and a drop-down drawn under the
//! region could not be clicked without the click reaching the region first. The bar and the
//! drop-down are drawn by the menu; what stays here is the order between them and the region.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, Widget};

use crate::about_dialog;
use crate::state::AppState;
use crate::{context::AppGlobal, menu};

pub(crate) const MENU_BAR_HEIGHT: u16 = 1;

pub(crate) fn draw(area: Rect, buf: &mut Buffer, state: &mut AppState, global: &AppGlobal) {
    let [bar_area, body_area] =
        Layout::vertical([Constraint::Length(MENU_BAR_HEIGHT), Constraint::Fill(1)]).areas(area);

    let title = if global.cfg.counter {
        format!("{} {}", global.cfg.title, state.counter)
    } else {
        global.cfg.title.clone()
    };

    // No title on the bar: the region's title already names the application, and a bar that
    // repeated it would say the same word twice on one frame.
    menu::draw_bar(bar_area, buf, &mut state.menu);
    Block::bordered().title(title).render(body_area, buf);
    menu::draw_popup(bar_area, buf, &mut state.menu);
    about_dialog::draw(area, buf, &mut state.about);
}

#[cfg(test)]
mod tests {
    use super::draw;
    use crate::config::Config;
    use crate::context::AppGlobal;
    use crate::paint::MENU_BAR_HEIGHT;
    use crate::state::AppState;
    use monospace_diagram::Diagram;
    use ratatui::{buffer::Buffer, layout::Rect};

    fn render_with(width: u16, height: u16, cfg: Config, state: &mut AppState) -> Buffer {
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
        draw(
            Rect::new(0, 0, width, height),
            &mut buffer,
            state,
            &AppGlobal::new(cfg),
        );
        buffer
    }

    fn render(width: u16, height: u16, cfg: Config) -> Buffer {
        render_with(width, height, cfg, &mut AppState::new(Diagram::new()))
    }

    fn row(buffer: &Buffer, y: u16) -> String {
        (buffer.area.left()..buffer.area.right())
            .map(|x| buffer[(x, y)].symbol().to_string())
            .collect()
    }

    fn all_rows(buffer: &Buffer) -> String {
        (buffer.area.top()..buffer.area.bottom())
            .map(|y| row(buffer, y))
            .collect::<Vec<String>>()
            .join("\n")
    }

    #[test]
    fn the_bar_is_the_first_thing_on_the_frame_and_the_region_is_below_it() {
        let buffer = render(30, 8, Config::default());
        let bar = row(&buffer, 0);
        let region_top = row(&buffer, MENU_BAR_HEIGHT);

        assert!(bar.contains("File"), "the bar has no File on it: {bar:?}");
        assert!(
            region_top.starts_with('┌'),
            "the region does not start under the bar: {region_top:?}"
        );
    }

    #[test]
    fn the_view_draws_a_titled_block_below_the_bar() {
        let buffer = render(20, 6, Config::default());
        let region_top = row(&buffer, MENU_BAR_HEIGHT);
        let region_bottom = row(&buffer, 5);

        assert_eq!(buffer[(0, MENU_BAR_HEIGHT)].symbol(), "┌");
        assert_eq!(buffer[(19, MENU_BAR_HEIGHT)].symbol(), "┐");
        assert_eq!(buffer[(0, 5)].symbol(), "└");
        assert_eq!(buffer[(19, 5)].symbol(), "┘");
        assert!(region_top.contains("monospace 0"), "{region_top:?}");
        assert!(region_bottom.contains('─'), "{region_bottom:?}");
    }

    #[test]
    fn the_configuration_reaches_the_frame() {
        let cfg = Config {
            title: "probe".to_string(),
            counter: false,
        };
        let buffer = render(40, 6, cfg);
        let region_top = row(&buffer, MENU_BAR_HEIGHT);
        let frame = all_rows(&buffer);

        assert!(
            region_top.contains("probe"),
            "the configured title is absent: {region_top:?}"
        );
        assert!(
            !frame.contains('0'),
            "the counter is drawn although it is off: {frame:?}"
        );
    }

    #[test]
    fn an_open_drop_down_is_drawn_over_the_region() {
        let mut state = AppState::new(Diagram::new());
        render_with(40, 8, Config::default(), &mut state);
        state.menu.bar.select(Some(0));
        state.menu.set_popup_active(true);

        let buffer = render_with(40, 8, Config::default(), &mut state);
        let frame = all_rows(&buffer);

        assert!(
            frame.contains("Quit"),
            "the drop-down shows nothing: {frame:?}"
        );
        assert!(
            !frame.contains("monospace 0"),
            "the region was pushed down instead of covered: {frame:?}"
        );
    }
}
