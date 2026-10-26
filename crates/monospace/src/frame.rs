//! One frame of the screen: the three regions, and the order they are painted in.
//!
//! **The order is the picture.** The bar, then the region, then the status bar, then the
//! drop-down, and the dialog last of all: a widget drawn later covers the one before it, and a
//! drop-down drawn under the region could not be seen, let alone clicked. **The dialog is the
//! overlay** (D6 of #189), so it covers the drop-down as well as everything else.
//!
//! **Every cell of the screen is painted on every frame.** The terminal diffs against the
//! previous frame rather than clearing, so a cell that is not painted keeps what was in it. The
//! frame therefore clears the screen before it draws anything into it, and a region that the
//! frame is not using — because the screen is too short to hold it — is cleared rather than
//! left alone.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::widgets::Widget;
use ratatui_widgets::clear::Clear;

use crate::context::AppGlobal;
use crate::state::{AppState, Focus};
use crate::{about, diagram_widget, menu, status};

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

    menu::draw_bar(
        regions.bar,
        buf,
        &mut state.menu,
        &global.cfg.theme,
        state.focus == Focus::Bar,
    );

    diagram_widget::draw(
        regions.region,
        buf,
        &mut state.diagram,
        &global.catalog,
        &global.cfg.theme,
    );

    status::draw(regions.status, buf, &mut state.status, &global.cfg.theme);

    // The drop-down is drawn after the region: it hangs from the bar and covers the region, and
    // a region painted after it would erase it.
    menu::draw_drop_down(area, buf, &mut state.menu, &global.cfg.theme);

    // **The dialog is drawn last of all** (D6): it is the overlay, so it covers the drop-down
    // and everything else, and a control behind it is not there to be seen or clicked.
    about::draw(regions.region, buf, &mut state.about, &global.cfg.theme);
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
    use crate::vocabulary::Action;
    use monospace_core::{Offset, ShapeId};
    use rat_theme4::palettes::dark::BASE16;
    use rat_theme4::themes::create_dark;
    use ratatui_core::buffer::Buffer;
    use ratatui_core::style::Modifier;
    use ratatui_crossterm::crossterm::event::{
        Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };
    use std::num::NonZeroU16;

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
        #[allow(
            clippy::default_trait_access,
            reason = "the diagram's type is not named here: the application depends on \
            the editing layer and not on the model below it"
        )]
        let session = monospace_editing::Session::new(Default::default());

        AppState::new(session)
    }

    /// The state of an application over the diagram `json` describes.
    fn a_state_over(json: &str) -> AppState {
        let diagram =
            monospace_description::parse(json).expect("the test's description is readable");

        AppState::new(monospace_editing::Session::new(diagram))
    }

    /// A description whose only shape is a light 4×3 box at `(2, 1)` of the window.
    const A_BOX: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 },
                      "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
    }"#;

    /// A description whose only shape is a 2×2 light box at `(2, 1)` of the window.
    const A_NARROW_BOX: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 },
                      "size": { "width": 2, "height": 2 }, "stroke": "light" } ]
    }"#;

    /// Two light 4×3 boxes side by side, the left at `(2, 1)` of the window and the right at
    /// `(6, 1)`: the second is as close to the first as a shape can be without sharing a cell.
    const A_BOX_AND_ITS_NEIGHBOR: &str = r#"{
        "next_id": 3,
        "shapes": [ { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 },
                      "size": { "width": 4, "height": 3 }, "stroke": "light" },
                    { "kind": "box", "id": 2, "at": { "x": 6, "y": 1 },
                      "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
    }"#;

    /// A description whose only shape is a 4×3 box in the ASCII table.
    const AN_ASCII_BOX: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                      "size": { "width": 4, "height": 3 }, "stroke": "ascii" } ]
    }"#;

    /// A description whose only shape names a stroke no table answers.
    const A_BOX_IN_NO_TABLE: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                      "size": { "width": 4, "height": 3 }, "stroke": "mystery" } ]
    }"#;

    /// The configuration the application starts with.
    fn a_global() -> AppGlobal {
        AppGlobal::new(Config::default(), monospace_glyph_sets::all())
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

    /// The drop-down's own cells, cut out of the frame.
    ///
    /// **The pop-up is drawn over what it covers**, so a picture of the frame holds the region
    /// behind it, and two frames of the same menu over two drawings would differ in the part the
    /// menu does not own. The cut is the pop-up's box: from the row under the bar to the row of its
    /// lower border, and from its upper-left corner to its upper-right one.
    fn drop_down_picture(buffer: &Buffer) -> String {
        let top = BAR_HEIGHT;
        let row: Vec<char> = row_of(buffer, top).chars().collect();

        // The region draws a corner of its own on this row, so the drop-down's is the last one.
        let mut left = None;

        for (column, glyph) in row.iter().enumerate() {
            if *glyph == '┌' {
                left = Some(column);
            }
        }

        let left = left.expect("the drop-down has no upper-left corner");
        let right = row
            .iter()
            .skip(left)
            .position(|glyph| *glyph == '┐')
            .map(|offset| left + offset)
            .expect("the drop-down has no upper-right corner");

        let mut lines = Vec::new();

        for line in top..buffer.area.bottom() {
            let text: String = row_of(buffer, line)
                .chars()
                .skip(left)
                .take(right - left + 1)
                .collect();
            let last = text.starts_with('└');

            lines.push(text);

            if last {
                break;
            }
        }

        lines.join("\n")
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

    /// Opens the drop-down of `navigation`, the way a person does.
    ///
    /// **The bar is drawn first**: the widget knows where its labels are only once a frame has
    /// been drawn, and a key that arrives before that finds nothing to open.
    fn open_the_drop_down(state: &mut AppState, global: &AppGlobal, navigation: char) {
        let _ = paint(state, global, WIDTH, HEIGHT);
        state.focus = Focus::Bar;
        let event = Event::Key(KeyEvent::new(KeyCode::Char(navigation), KeyModifiers::NONE));
        let _ = crate::menu::handle(&event, &mut state.menu, &mut state.focus);
        assert!(
            state.menu.popup_active(),
            "the navigation character did not open the drop-down"
        );
    }

    /// The picture of a frame, with the counts under it.
    fn described(buffer: &Buffer, regions: Regions) -> String {
        format!("{}\n{}", picture(buffer), counts(buffer, regions))
    }

    /// Whether the cell at a frame position is painted in reverse video.
    fn reversed(buffer: &Buffer, column: u16, line: u16) -> bool {
        buffer[(column, line)]
            .style()
            .add_modifier
            .contains(Modifier::REVERSED)
    }

    /// The frame positions whose cell is painted in reverse video, in the order the frame is read.
    fn inverted(buffer: &Buffer) -> Vec<(u16, u16)> {
        (buffer.area.top()..buffer.area.bottom())
            .flat_map(|line| {
                (buffer.area.left()..buffer.area.right()).map(move |column| (column, line))
            })
            .filter(|(column, line)| reversed(buffer, *column, *line))
            .collect()
    }

    /// The screen positions the box of `A_BOX` writes: its border, at `(2, 1)` of the window and
    /// four cells wide and three tall.
    ///
    /// **A box's interior is not a cell of the drawing**, so nothing is written there and nothing
    /// is inverted there: the ten cells here are what the copy writes for the box. The window's
    /// corner is the screen's `(1, 2)`.
    fn cells_of_the_box() -> Vec<(u16, u16)> {
        vec![
            (3, 3),
            (4, 3),
            (5, 3),
            (6, 3),
            (3, 4),
            (6, 4),
            (3, 5),
            (4, 5),
            (5, 5),
            (6, 5),
        ]
    }

    /// Clicks the screen position `(column, line)` with the left button, the way a person does.
    fn click(state: &mut AppState, column: u16, line: u16) {
        let event = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row: line,
            modifiers: KeyModifiers::NONE,
        });

        let outcome = crate::diagram_widget::handle(&event, &mut state.diagram);

        assert!(
            matches!(outcome, Some(Action::ScreenChanged)),
            "({column}, {line}) selected nothing: {outcome:?}"
        );
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
    fn the_interior_is_the_window_the_diagram_is_drawn_into() {
        let mut state = a_state_over(A_BOX);
        let buffer = paint(&mut state, &a_global(), WIDTH, HEIGHT);

        // The box is at `(2, 1)` of the window, and the window starts one cell in from the
        // region's corner: the region's row is 1, so the box's top-left is row 3, column 3.
        assert_eq!(
            buffer[(3, 3)].symbol(),
            "┌",
            "the box's top-left is not at the interior's (2, 1)"
        );
        assert_eq!(
            buffer[(6, 3)].symbol(),
            "┐",
            "the box is not four cells wide from the interior's corner"
        );
        assert_eq!(
            buffer[(3, 5)].symbol(),
            "└",
            "the box is not three cells tall from the interior's corner"
        );
        assert_eq!(
            buffer[(1, 2)].symbol(),
            " ",
            "the interior's own corner holds something"
        );

        // The record counts the offset from the same corner the drawing does.
        assert!(
            state.diagram.owner(Offset { x: 2, y: 1 }).is_some(),
            "the box does not own the cell it drew"
        );
    }

    #[test]
    fn the_diagram_is_drawn_again_on_every_frame() {
        let mut state = a_state_over(A_BOX);
        let global = a_global();

        let _ = paint(&mut state, &global, WIDTH, HEIGHT);
        assert!(
            state.diagram.owner(Offset { x: 2, y: 2 }).is_some(),
            "the first frame did not draw the box"
        );

        // A screen three rows shorter: the interior is two rows tall, and the box's third row
        // is outside it. The drawing of the frame that follows is what answers.
        let _ = paint(&mut state, &global, WIDTH, 6);
        assert!(
            state.diagram.owner(Offset { x: 2, y: 2 }).is_none(),
            "the drawing of the first frame survived the second"
        );
    }

    #[test]
    fn a_cell_no_shape_wrote_is_a_space() {
        let buffer = paint(&mut a_state_over(A_BOX), &a_global(), WIDTH, HEIGHT);

        assert_eq!(
            buffer[(1, 2)].symbol(),
            " ",
            "the interior's corner, which no shape wrote, is not a space"
        );
    }

    #[test]
    fn a_glyph_the_catalog_does_not_know_is_a_space() {
        let buffer = paint(
            &mut a_state_over(A_BOX_IN_NO_TABLE),
            &a_global(),
            WIDTH,
            HEIGHT,
        );

        // The box's own cells are at `(0, 0)` through `(3, 2)` of the window, and no table has a
        // rule for its stroke: every one of them renders as a space.
        for line in 2..5 {
            for column in 1..5 {
                assert_eq!(
                    buffer[(column, line)].symbol(),
                    " ",
                    "({column}, {line}) is not a space"
                );
            }
        }
    }

    #[test]
    fn the_owner_survives_the_copy() {
        let mut state = a_state_over(A_BOX);
        let buffer = paint(&mut state, &a_global(), WIDTH, HEIGHT);
        let diagram = &state.diagram;

        let top_left = Offset { x: 2, y: 1 };
        let top_right = Offset { x: 5, y: 1 };

        assert!(
            diagram.owner(top_left).is_some(),
            "the copy took the record with it"
        );
        assert_eq!(
            diagram.owner(top_left),
            diagram.owner(top_right),
            "two cells of the same box name two different shapes"
        );

        // The picture and the record are the same drawing.
        assert_eq!(buffer[(3, 3)].symbol(), "┌", "the copy drew no box");
        assert_eq!(buffer[(6, 3)].symbol(), "┐", "the copy drew no box");
    }

    #[test]
    fn the_owner_is_asked_with_the_click_offset_and_nothing_added() {
        let mut state = a_state_over(A_NARROW_BOX);
        let _ = paint(&mut state, &a_global(), WIDTH, HEIGHT);
        let diagram = &state.diagram;

        // The box is at `(2, 1)` of the window and covers two rows: `(3, 3)` is the click with
        // the interior's corner `(1, 2)` added to it, and it names a cell nobody wrote.
        assert!(
            diagram.owner(Offset { x: 2, y: 1 }).is_some(),
            "the click's own offset is not the one asked"
        );
        assert!(
            diagram.owner(Offset { x: 3, y: 3 }).is_none(),
            "the interior's corner was added to the click"
        );
    }

    #[test]
    fn the_buffer_of_the_last_frame_answers_a_click() {
        let mut state = a_state_over(A_BOX);
        let global = a_global();

        assert!(
            state.diagram.owner(Offset { x: 2, y: 1 }).is_none(),
            "the empty drawing answered a click"
        );

        let _ = paint(&mut state, &global, WIDTH, HEIGHT);
        let asked = state.diagram.owner(Offset { x: 2, y: 1 });
        assert!(asked.is_some(), "the frame's drawing does not answer");

        let _ = paint(&mut state, &global, WIDTH, HEIGHT);
        assert_eq!(
            state.diagram.owner(Offset { x: 2, y: 1 }),
            asked,
            "the second frame's drawing does not answer the same click"
        );
    }

    /// **A click is a question and not a command.** The session is only read, so the frame that
    /// follows holds the same cells and the same record; what the click changed is the style of
    /// those cells, which a picture of symbols does not show.
    #[test]
    fn a_click_changes_no_diagram() {
        let mut state = a_state_over(A_BOX);
        let global = a_global();

        let before = paint(&mut state, &global, WIDTH, HEIGHT);
        let asked = state.diagram.owner(Offset { x: 2, y: 1 });
        assert!(asked.is_some(), "the box is not in the drawing");

        // The box's top-left is the screen's `(3, 3)`.
        click(&mut state, 3, 3);

        let after = paint(&mut state, &global, WIDTH, HEIGHT);

        assert_eq!(
            state.diagram.owner(Offset { x: 2, y: 1 }),
            asked,
            "the click changed which shape the drawing records"
        );
        assert_eq!(
            picture(&before),
            picture(&after),
            "the click changed the cells the frame holds"
        );
        assert!(
            !inverted(&after).is_empty(),
            "the click selected nothing, so the frame proves nothing"
        );
    }

    /// **The inversion is the record's answer, cell by cell.** The cells of the selected shape are
    /// reverse video and no others, however close another shape's cells are.
    #[test]
    fn the_selected_shapes_cells_are_inverted_and_no_others() {
        let mut state = a_state_over(A_BOX_AND_ITS_NEIGHBOR);
        let global = a_global();
        let _ = paint(&mut state, &global, WIDTH, HEIGHT);

        // The left box's top-left, and no cell of the right one.
        click(&mut state, 3, 3);
        let buffer = paint(&mut state, &global, WIDTH, HEIGHT);

        assert_eq!(
            inverted(&buffer),
            cells_of_the_box(),
            "the cells the frame inverts are not the selected box's"
        );
        assert!(
            !reversed(&buffer, 7, 3),
            "the neighboring box's border is inverted"
        );
    }

    /// **The inversion is read from the record and not from the selection.** An identity no cell of
    /// the drawing names inverts nothing, which is what leaves a command that removes the selected
    /// shape a frame with no inversion rather than a stale one (rule 7).
    #[test]
    fn a_selection_the_drawing_does_not_record_inverts_nothing() {
        let mut state = a_state_over(A_BOX);
        let global = a_global();
        let _ = paint(&mut state, &global, WIDTH, HEIGHT);

        let unrecorded = ShapeId::new(NonZeroU16::new(9).expect("9 is not zero"));
        assert!(
            state.diagram.select(Some(unrecorded)),
            "the selection did not move"
        );

        let buffer = paint(&mut state, &global, WIDTH, HEIGHT);

        assert!(
            inverted(&buffer).is_empty(),
            "an identity no cell records inverted cells"
        );
    }

    /// **A shape the window does not hold is still selected.** The state holds the identity, so a
    /// frame that holds the shape again inverts it: the selection is what the click recorded, and
    /// not the cells the frame that answered it happened to hold.
    #[test]
    fn a_selected_shape_with_no_cell_in_the_window_is_still_selected() {
        let mut state = a_state_over(A_BOX);
        let global = a_global();
        let _ = paint(&mut state, &global, WIDTH, HEIGHT);
        click(&mut state, 3, 3);

        // Four rows leave the region two and its interior none: the window holds no cell of the
        // box, so nothing is drawn and nothing is inverted.
        let empty = paint(&mut state, &global, WIDTH, 4);
        assert!(
            inverted(&empty).is_empty(),
            "a cell of a shape the window does not hold is inverted"
        );
        assert!(
            state.diagram.selected().is_some(),
            "the selection was dropped when the window stopped holding the shape"
        );

        let held_again = paint(&mut state, &global, WIDTH, HEIGHT);

        assert_eq!(
            inverted(&held_again),
            cells_of_the_box(),
            "the frame that holds the box again does not invert it"
        );
    }

    /// **The selection is an identity, so a resize recomputes what it covers.** A screen shorter
    /// than the box clips it: the part the window holds is inverted and the rest is not, and the
    /// screen that holds the whole box again inverts the whole box.
    #[test]
    fn the_selection_survives_a_resize() {
        let mut state = a_state_over(A_BOX);
        let global = a_global();
        let _ = paint(&mut state, &global, WIDTH, HEIGHT);
        click(&mut state, 3, 3);

        // Seven rows leave the region five rows and its interior three: the box's third row is
        // outside the window.
        let clipped = paint(&mut state, &global, WIDTH, 7);
        let rows_the_window_holds: Vec<(u16, u16)> = cells_of_the_box()
            .into_iter()
            .filter(|(_, line)| *line < 5)
            .collect();

        assert_eq!(
            inverted(&clipped),
            rows_the_window_holds,
            "the resize inverted cells the window does not hold, or dropped the cells it does"
        );

        let restored = paint(&mut state, &global, WIDTH, HEIGHT);

        assert_eq!(
            inverted(&restored),
            cells_of_the_box(),
            "the selection did not survive the resize"
        );
    }

    /// **The dialog is drawn last of all** (rule 9): it is the overlay, so it covers the drop-down
    /// as well as the region, and a control behind it is not there to be seen.
    #[test]
    fn the_dialog_is_drawn_last_of_all() {
        let mut state = a_state();
        let global = a_global();

        open_the_drop_down(&mut state, &global, 'f');
        about::open(&mut state.about);
        let frame = picture(&paint(&mut state, &global, WIDTH, HEIGHT));

        assert!(
            frame.contains(&format!("About {}", status::NAME)),
            "the dialog is not on the frame: {frame:?}"
        );
        assert!(
            !frame.contains("Quit"),
            "the drop-down is drawn over the dialog: {frame:?}"
        );
    }

    /// **The palette is the dark BASE16 theme** (rule 12): the frame the default configuration
    /// paints is the one the theme paints, cell for cell, so nothing reads a terminal color and no
    /// other palette is in use.
    #[test]
    fn the_palette_is_the_dark_base16_theme() {
        let by_default = paint(&mut a_state(), &a_global(), WIDTH, HEIGHT);

        let dark = AppGlobal::new(
            Config {
                theme: create_dark(BASE16),
            },
            monospace_glyph_sets::all(),
        );
        let by_the_theme = paint(&mut a_state(), &dark, WIDTH, HEIGHT);

        for line in 0..HEIGHT {
            for column in 0..WIDTH {
                assert_eq!(
                    by_default[(column, line)].symbol(),
                    by_the_theme[(column, line)].symbol(),
                    "({column}, {line}) is not what the dark BASE16 theme paints"
                );
                assert_eq!(
                    by_default[(column, line)].style(),
                    by_the_theme[(column, line)].style(),
                    "({column}, {line}) is painted in another palette"
                );
            }
        }
    }

    /// **The region has no focus cue of its own** (D2 of #189): it is the body, and it is drawn the
    /// same whether or not it holds the keyboard. A cue lives on a control that has one, so the
    /// frame that moves the keyboard to the bar leaves every cell of the region untouched.
    #[test]
    fn the_region_is_drawn_the_same_with_and_without_the_keyboard() {
        let mut state = a_state_over(A_BOX);
        let global = a_global();
        let area = Rect::new(0, 0, WIDTH, HEIGHT);
        let region = regions(area).region;

        state.focus = Focus::Region;
        let without = paint(&mut state, &global, WIDTH, HEIGHT);
        state.focus = Focus::Bar;
        let with = paint(&mut state, &global, WIDTH, HEIGHT);

        for line in region.top()..region.bottom() {
            for column in region.left()..region.right() {
                assert_eq!(
                    with[(column, line)].symbol(),
                    without[(column, line)].symbol(),
                    "({column}, {line}) changed with the keyboard"
                );
                assert_eq!(
                    with[(column, line)].style(),
                    without[(column, line)].style(),
                    "({column}, {line}) was painted differently with the keyboard"
                );
            }
        }
    }

    #[test]
    fn an_ascii_box_is_drawn_with_the_ascii_table() {
        let buffer = paint(&mut a_state_over(AN_ASCII_BOX), &a_global(), WIDTH, HEIGHT);

        assert!(
            row_of(&buffer, 2).starts_with("│+--+"),
            "the box's top is not in the ASCII table: {:?}",
            row_of(&buffer, 2)
        );
        assert!(
            row_of(&buffer, 3).starts_with("│|  |"),
            "the box's sides are not in the ASCII table: {:?}",
            row_of(&buffer, 3)
        );
        assert!(
            row_of(&buffer, 4).starts_with("│+--+"),
            "the box's bottom is not in the ASCII table: {:?}",
            row_of(&buffer, 4)
        );
    }

    #[test]
    fn an_empty_diagram_draws_nothing() {
        let buffer = paint(&mut a_state(), &a_global(), WIDTH, HEIGHT);
        let region = regions(Rect::new(0, 0, WIDTH, HEIGHT)).region;

        for line in region.top() + 1..region.bottom() - 1 {
            for column in region.left() + 1..region.right() - 1 {
                assert_eq!(
                    buffer[(column, line)].symbol(),
                    " ",
                    "({column}, {line}) is not blank"
                );
            }
        }

        assert!(
            row_of(&buffer, region.top()).starts_with('┌'),
            "the frame went with the diagram"
        );
    }

    #[test]
    fn a_region_with_no_interior_draws_nothing() {
        let mut state = a_state_over(A_BOX);
        let buffer = paint(&mut state, &a_global(), WIDTH, 3);

        assert_eq!(
            row_of(&buffer, 1),
            " ".repeat(usize::from(WIDTH)),
            "a region with no interior drew something"
        );
        assert!(
            state.diagram.owner(Offset { x: 0, y: 0 }).is_none(),
            "a window with no cells owns one"
        );
    }

    #[test]
    fn the_demonstration_is_drawn_when_no_path_is_given() {
        let session = crate::start(None).expect("the shipped demonstration is readable");
        let buffer = paint(&mut AppState::new(session), &a_global(), WIDTH, HEIGHT);

        let first = row_of(&buffer, 2);
        assert!(
            first.starts_with("│┌──┐"),
            "the demonstration's first row is not inside the frame: {first:?}"
        );
        assert!(
            first.contains("+--+"),
            "the demonstration's ASCII box is not drawn: {first:?}"
        );
    }

    #[test]
    fn the_description_at_the_path_is_drawn() {
        let path =
            std::env::temp_dir().join(format!("monospace-app-frame-{}.json", std::process::id()));
        std::fs::write(&path, A_BOX).expect("the temp description should be writable");
        let named = path.to_str().expect("the temp path is UTF-8");

        let session = crate::start(Some(named)).expect("the description at the path is readable");
        let buffer = paint(&mut AppState::new(session), &a_global(), WIDTH, HEIGHT);

        let _ = std::fs::remove_file(&path);

        assert_eq!(
            buffer[(3, 3)].symbol(),
            "┌",
            "the description at the path is not drawn where its box says"
        );
    }

    #[test]
    fn the_screen_with_the_demo() {
        let session = crate::start(None).expect("the shipped demonstration is readable");
        let mut state = AppState::new(session);
        let area = Rect::new(0, 0, WIDTH, HEIGHT);
        let buffer = paint(&mut state, &a_global(), WIDTH, HEIGHT);

        snap(
            "the_screen_with_the_demo",
            &format!(
                "the screen with the demonstration, {WIDTH}x{HEIGHT}\n{}",
                described(&buffer, regions(area))
            ),
        );
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

        open_the_drop_down(&mut state, &global, 'f');
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
        let mut state = a_state_over(A_BOX);
        let global = a_global();

        let before = paint(&mut state, &global, WIDTH, HEIGHT);

        // The drop-down opens over the region and closes again.
        open_the_drop_down(&mut state, &global, 'f');
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

    /// **The entries do not move with the history** (rule 10). The same drop-down is drawn over a
    /// fresh session and over one a deletion has left, and the two pictures are equal: no entry
    /// carries a flag, and `can_undo` and `can_redo` are not read by the frame.
    #[test]
    fn the_edit_menu_offers_undo_and_redo_at_every_position() {
        let mut state = a_state_over(A_BOX);
        let global = a_global();

        open_the_drop_down(&mut state, &global, 'e');
        let fresh = drop_down_picture(&paint(&mut state, &global, WIDTH, HEIGHT));

        // The drop-down closes, the box is removed, and the same drop-down opens over the newest
        // position.
        state.menu.set_popup_active(false);
        let shape = ShapeId::new(NonZeroU16::new(1).expect("1 is not zero"));
        state.diagram.remove(shape);
        let _ = paint(&mut state, &global, WIDTH, HEIGHT);
        open_the_drop_down(&mut state, &global, 'e');
        let after_the_delete = drop_down_picture(&paint(&mut state, &global, WIDTH, HEIGHT));

        assert!(
            fresh.contains("Undo"),
            "the drop-down shows no undo: {fresh:?}"
        );
        assert!(
            fresh.contains("Redo"),
            "the drop-down shows no redo: {fresh:?}"
        );
        assert_eq!(
            fresh, after_the_delete,
            "the Edit menu moves with the history"
        );
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

        open_the_drop_down(&mut state, &global, 'f');
        let buffer = paint(&mut state, &global, WIDTH, HEIGHT);

        snap(
            "the_drop_down_open_over_the_region",
            &format!(
                "the drop-down open over the region, {WIDTH}x{HEIGHT}\n{}",
                described(&buffer, regions(area))
            ),
        );
    }

    /// **The Edit drop-down is the picture the two extra entries make.** Undo, Redo and Delete are
    /// drawn in that order, each with the shortcut the table answers to.
    #[test]
    fn the_edit_drop_down() {
        let mut state = a_state();
        let global = a_global();
        let area = Rect::new(0, 0, WIDTH, HEIGHT);

        open_the_drop_down(&mut state, &global, 'e');
        let buffer = paint(&mut state, &global, WIDTH, HEIGHT);

        snap(
            "the_edit_drop_down",
            &format!(
                "the Edit drop-down open over the region, {WIDTH}x{HEIGHT}\n{}",
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
