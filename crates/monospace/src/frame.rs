//! One frame of the screen: the three regions, and the order they are painted in.
//!
//! **The order is the picture.** The bar, then the region, then the status bar, and the
//! drop-down last of all: a widget drawn later covers the one before it, and a drop-down drawn
//! under the region could not be seen, let alone clicked.
//!
//! **Every cell of the screen is painted on every frame.** The terminal diffs against the
//! previous frame rather than clearing, so a cell that is not painted keeps what was in it. The
//! frame therefore clears the screen before it draws anything into it, and a region that the
//! frame is not using — because the screen is too short to hold it — is cleared rather than
//! left alone.

use rat_theme4::StyleName;
use rat_theme4::theme::SalsaTheme;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::widgets::Widget;
use ratatui_widgets::block::Block;
use ratatui_widgets::clear::Clear;

use crate::context::AppGlobal;
use crate::state::AppState;
use crate::{menu, status};

/// The height of the bar across the top.
pub(crate) const BAR_HEIGHT: u16 = 1;

/// The height of the status bar along the bottom.
pub(crate) const STATUS_HEIGHT: u16 = 1;

/// The three regions of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Regions {
    /// The bar across the top.
    pub bar: Rect,

    /// The region the diagram goes into.
    pub region: Rect,

    /// The status bar along the bottom.
    pub status: Rect,
}

/// The regions of a screen of the given size.
///
/// **The bar's row is the bar's and the status bar's row is the status bar's**, whatever the
/// size of the screen: what is left between them is the region, and it takes the rest.
pub(crate) fn regions(area: Rect) -> Regions {
    let [bar, region, status] = Layout::vertical([
        Constraint::Length(BAR_HEIGHT),
        Constraint::Fill(1),
        Constraint::Length(STATUS_HEIGHT),
    ])
    .areas(area);

    Regions {
        bar,
        region,
        status,
    }
}

/// Paints one frame.
pub(crate) fn draw(area: Rect, buf: &mut Buffer, state: &mut AppState, global: &AppGlobal) {
    let regions = regions(area);

    // Every cell of the screen is painted on every frame, and this is what paints the ones no
    // region uses.
    Clear.render(area, buf);

    menu::draw_bar(regions.bar, buf, &mut state.menu, &global.cfg.theme);
    draw_region(regions.region, buf, &global.cfg.theme);
    status::draw(regions.status, buf, &mut state.status, &global.cfg.theme);

    // The drop-down is drawn last: it hangs from the bar and covers the region, and a region
    // painted after it would erase it.
    menu::draw_drop_down(area, buf, &mut state.menu, &global.cfg.theme);
}

/// Paints the region the diagram goes into.
///
/// **It is a frame and nothing else.** What is inside it is the diagram's, and the diagram is
/// #175's; all this slice draws is the box it goes into.
///
/// A box needs two rows, one for its top and one for its bottom, and a region that is shorter
/// than that is not drawn at all: the frame clears it, and what a previous frame left in it is
/// gone.
fn draw_region(area: Rect, buf: &mut Buffer, theme: &SalsaTheme) {
    if area.height < 2 {
        return;
    }

    let border: Style = theme.style(Style::CONTAINER_BORDER_FG);
    Block::bordered().style(border).render(area, buf);
}

/// Where a piece of text is on a frame, found in the frame itself so that a growing label
/// does not move a click away from it.
#[cfg(test)]
pub(crate) fn cell_of(buffer: &Buffer, text: &str) -> (u16, u16) {
    (buffer.area.top()..buffer.area.bottom())
        .find_map(|line| {
            let row: String = (buffer.area.left()..buffer.area.right())
                .map(|column| buffer[(column, line)].symbol().to_string())
                .collect();

            row.find(text).map(|offset| {
                let column =
                    buffer.area.left() + u16::try_from(offset).expect("an offset fits a frame");
                (column, line)
            })
        })
        .unwrap_or_else(|| panic!("{text} is not on the frame"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::context::AppGlobal;
    use crate::state::AppState;
    use crate::status;
    use ratatui_core::buffer::Buffer;
    use ratatui_crossterm::crossterm::event::{Event, KeyCode, KeyEvent};

    /// The width of the screens these tests draw.
    const WIDTH: u16 = 40;

    /// The height of the screens these tests draw.
    const HEIGHT: u16 = 10;

    /// What one snapshot says about itself, so that a reader of the file knows what it is
    /// looking at without opening the code.
    const WHAT: &str = concat!(
        "A picture of the screen, as the frame paints it. The three regions are counted under ",
        "the picture: the bar across the top, the region the diagram goes into, and the status ",
        "bar along the bottom. The count of the cells that are not blank is there because a ",
        "picture alone cannot say how much of the screen is spoken for, and because a number ",
        "that is derived from the frame cannot be wrong the way a hand-written expectation is.",
    );

    /// The state of an application that starts with nothing in it.
    fn a_state() -> AppState {
        AppState::new()
    }

    /// The configuration the application starts with.
    fn a_global() -> AppGlobal {
        AppGlobal::new(Config::default())
    }

    /// Paints a frame into a buffer of the given size.
    fn paint(state: &mut AppState, global: &AppGlobal, width: u16, height: u16) -> Buffer {
        let area = Rect::new(0, 0, width, height);
        let mut buffer = Buffer::empty(area);
        draw(area, &mut buffer, state, global);
        buffer
    }

    /// One row of a frame, as text.
    fn row_of(buffer: &Buffer, line: u16) -> String {
        (buffer.area.left()..buffer.area.right())
            .map(|column| buffer[(column, line)].symbol().to_string())
            .collect()
    }

    /// The picture of a frame, as text, with the trailing blanks of each row cut.
    fn picture(buffer: &Buffer) -> String {
        (buffer.area.top()..buffer.area.bottom())
            .map(|line| row_of(buffer, line).trim_end().to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// The number of cells of the frame that are not blank.
    fn written(buffer: &Buffer) -> usize {
        (buffer.area.top()..buffer.area.bottom())
            .flat_map(|line| {
                (buffer.area.left()..buffer.area.right()).map(move |column| (line, column))
            })
            .filter(|(line, column)| buffer[(*column, *line)].symbol() != " ")
            .count()
    }

    /// The counts of a frame: how many rows each region has, and how many cells are not blank.
    fn counts(buffer: &Buffer, regions: Regions) -> String {
        let total = usize::from(buffer.area.width) * usize::from(buffer.area.height);

        format!(
            "the bar is {} {}, the region is {} {} and the status is {} {}\n\
             {} of {total} cells are not blank",
            regions.bar.height,
            rows(regions.bar.height),
            regions.region.height,
            rows(regions.region.height),
            regions.status.height,
            rows(regions.status.height),
            written(buffer),
        )
    }

    /// What a number of rows is called.
    fn rows(count: u16) -> &'static str {
        if count == 1 { "row" } else { "rows" }
    }

    /// Writes one snapshot.
    fn snap(name: &str, body: &str) {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/snapshots/contract"
        ));
        settings.set_description(WHAT);
        settings.bind(|| insta::assert_snapshot!(name, body));
    }

    /// Opens the drop-down of the bar, the way a person does.
    ///
    /// **The bar is drawn first**: the widget knows where its labels are only once a frame has
    /// been drawn, and a key that arrives before that finds nothing to open.
    fn open_the_drop_down(state: &mut AppState, global: &AppGlobal) {
        let _ = paint(state, global, WIDTH, HEIGHT);
        let _ = crate::menu::handle(&navigation_character(), &mut state.menu);
        assert!(
            state.menu.popup_active(),
            "the navigation character did not open the drop-down"
        );
    }

    /// The navigation character of the only label of the bar.
    fn navigation_character() -> Event {
        Event::Key(KeyEvent::new(
            KeyCode::Char('f'),
            ratatui_crossterm::crossterm::event::KeyModifiers::NONE,
        ))
    }

    /// The picture of a frame, with the counts under it.
    fn described(buffer: &Buffer, regions: Regions) -> String {
        format!("{}\n{}", picture(buffer), counts(buffer, regions))
    }

    #[test]
    fn the_bar_is_at_the_top_and_the_status_bar_is_at_the_bottom() {
        let buffer = paint(&mut a_state(), &a_global(), WIDTH, HEIGHT);

        let bar = row_of(&buffer, 0);
        let region = row_of(&buffer, 1);
        let status_bar = row_of(&buffer, HEIGHT - 1);

        assert!(bar.contains("File"), "the bar has no File on it: {bar:?}");
        assert!(
            region.starts_with('┌'),
            "the region does not start under the bar: {region:?}"
        );
        assert!(
            status_bar.contains(status::NAME),
            "the status bar says nothing: {status_bar:?}"
        );
    }

    #[test]
    fn the_region_between_them_takes_the_rest_of_the_screen() {
        for height in [3, 10, 24] {
            let regions = regions(Rect::new(0, 0, WIDTH, height));

            assert_eq!(regions.bar.height, BAR_HEIGHT, "{height} rows");
            assert_eq!(regions.status.height, STATUS_HEIGHT, "{height} rows");
            assert_eq!(
                regions.region.height,
                height - BAR_HEIGHT - STATUS_HEIGHT,
                "{height} rows"
            );
            assert_eq!(regions.region.top(), BAR_HEIGHT, "{height} rows");
            assert_eq!(
                regions.region.bottom(),
                height - STATUS_HEIGHT,
                "{height} rows"
            );
        }
    }

    #[test]
    fn the_bars_row_is_drawn_on_every_frame() {
        let mut state = a_state();
        let global = a_global();

        let at_rest = row_of(&paint(&mut state, &global, WIDTH, HEIGHT), 0);

        // The drop-down opens over the region, and the bar's row is still the bar's row.
        state.menu.bar.select(Some(0));
        state.menu.set_popup_active(true);
        let with_drop_down = row_of(&paint(&mut state, &global, WIDTH, HEIGHT), 0);

        assert!(
            at_rest.contains("File"),
            "the bar has no File on it: {at_rest:?}"
        );
        assert_eq!(at_rest, with_drop_down, "the drop-down took the bar's row");
    }

    #[test]
    fn a_region_that_stops_being_drawn_does_not_keep_what_was_last_in_it() {
        let mut state = a_state();
        let global = a_global();
        let mut buffer = Buffer::empty(Rect::new(0, 0, WIDTH, HEIGHT));

        // A screen tall enough for the region: the region is a box.
        draw(
            Rect::new(0, 0, WIDTH, HEIGHT),
            &mut buffer,
            &mut state,
            &global,
        );
        assert!(
            row_of(&buffer, 1).starts_with('┌'),
            "the region is not a box: {:?}",
            row_of(&buffer, 1)
        );

        // A screen too short to hold it: the region is not drawn, and what it held is gone.
        draw(Rect::new(0, 0, WIDTH, 3), &mut buffer, &mut state, &global);

        assert_eq!(
            row_of(&buffer, 1),
            " ".repeat(usize::from(WIDTH)),
            "the region kept what was last in it"
        );
    }

    #[test]
    fn a_drop_down_is_not_erased_by_the_region_being_painted_after_it() {
        let mut state = a_state();
        let global = a_global();

        open_the_drop_down(&mut state, &global);
        let buffer = paint(&mut state, &global, WIDTH, HEIGHT);
        let frame = picture(&buffer);

        assert!(
            frame.contains("Quit"),
            "the drop-down shows nothing: {frame:?}"
        );
        assert!(frame.contains('┐'), "the drop-down has no frame: {frame:?}");
    }

    #[test]
    fn a_drop_down_leaves_the_region_byte_identical_when_it_closes() {
        let mut state = a_state();
        let global = a_global();

        let before = paint(&mut state, &global, WIDTH, HEIGHT);

        // The drop-down opens over the region and closes again.
        open_the_drop_down(&mut state, &global);
        let _ = paint(&mut state, &global, WIDTH, HEIGHT);
        state.menu.set_popup_active(false);
        let after = paint(&mut state, &global, WIDTH, HEIGHT);

        for line in 0..HEIGHT {
            assert_eq!(
                row_of(&before, line),
                row_of(&after, line),
                "row {line} is not what it was"
            );
        }
    }

    #[test]
    fn the_screen_at_rest() {
        let mut state = a_state();
        let global = a_global();
        let area = Rect::new(0, 0, WIDTH, HEIGHT);
        let buffer = paint(&mut state, &global, WIDTH, HEIGHT);

        snap(
            "the_screen_at_rest",
            &format!(
                "the screen at rest, {WIDTH}x{HEIGHT}\n{}",
                described(&buffer, regions(area))
            ),
        );
    }

    #[test]
    fn the_drop_down_open_over_the_region() {
        let mut state = a_state();
        let global = a_global();
        let area = Rect::new(0, 0, WIDTH, HEIGHT);

        open_the_drop_down(&mut state, &global);
        let buffer = paint(&mut state, &global, WIDTH, HEIGHT);

        snap(
            "the_drop_down_open_over_the_region",
            &format!(
                "the drop-down open over the region, {WIDTH}x{HEIGHT}\n{}",
                described(&buffer, regions(area))
            ),
        );
    }

    #[test]
    fn the_status_bar_below_the_region() {
        let mut state = a_state();
        let global = a_global();
        let area = Rect::new(0, 0, WIDTH, HEIGHT);
        let buffer = paint(&mut state, &global, WIDTH, HEIGHT);

        let regions = regions(area);
        let status_bar = row_of(&buffer, HEIGHT - 1);

        assert!(
            regions.status.top() == regions.region.bottom(),
            "the status bar is not below the region"
        );
        assert!(
            status_bar.contains(status::NAME),
            "the status bar says nothing: {status_bar:?}"
        );

        snap(
            "the_status_bar_below_the_region",
            &format!(
                "the status bar below the region, {WIDTH}x{HEIGHT}\n{}",
                described(&buffer, regions)
            ),
        );
    }
}
