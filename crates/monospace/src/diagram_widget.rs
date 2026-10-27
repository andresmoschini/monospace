//! The region the diagram goes into, and the diagram inside it.
//!
//! **The box and what it holds are two things.** The box says where the diagram's place is and
//! comes from #174; the cells inside it are the diagram's, drawn into a window the size of the
//! interior and copied here one at a time.
//!
//! **The drawing is the state's, and the picture is this module's.** The buffer is rebuilt on
//! every frame at the scroll origin and kept in `DiagramState`, because a click is answered from
//! the picture the person saw rather than from a drawing made when the click arrives. See
//! _Drawing_ and _Ownership_ in `docs/diagram-model.md` §7 and §8.
//!
//! **A click is two questions and this module answers both.** Which shape decided the cell under
//! the pointer is the drawing's record, and what the screen shows about that answer is reverse
//! video painted on the cells the copy writes. The record keeps no style and the state keeps an
//! identity, so nothing below this module learns what a selection looks like.

use rat_theme4::StyleName;
use rat_theme4::theme::SalsaTheme;
use rat_widget::event::{HandleEvent, MouseOnly};
use rat_widget::scrolled::{Scroll, ScrollbarPolicy};
use ratatui_widgets::block::Block;

use monospace_core::{GlyphCatalog, Offset};
use ratatui_core::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Style,
    widgets::{StatefulWidget, Widget},
};
use ratatui_crossterm::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use crate::state::{DiagramState, Focus};
use crate::vocabulary::Action;

/// Paints the region, and the diagram the state draws inside it.
pub(crate) fn draw(
    area: Rect,
    buf: &mut Buffer,
    state: &mut DiagramState,
    catalog: &GlyphCatalog,
    theme: &SalsaTheme,
) {
    draw_region(area, buf, theme);
    let interior = inner_region(area);
    state.redraw(area, interior);
    draw_diagram(interior, buf, state, catalog);
    draw_bars(area, buf, state);
}

/// What an event means to the region.
///
/// **The bars are asked first, and the region after them** (D3): a wheel turn over the region and a
/// press or a drag on either bar are the bars' own answers, and only what they leave alone is a
/// question about the drawing.
///
/// **The region answers a click and the four arrows.** A click is the region's whatever leaf holds
/// the keyboard (D1), and a key is the region's only while it does: the arrows are what the region
/// does with the keyboard it is given.
///
/// **Everything else is answered with nothing** so that the event goes on to the next handler
/// (rule 3).
pub(crate) fn handle(raw: &Event, state: &mut DiagramState, focus: Focus) -> Option<Action> {
    if let Some(action) = scroll(raw, state) {
        return Some(action);
    }

    match raw {
        Event::Mouse(mouse) => click(*mouse, state),
        Event::Key(_) if focus == Focus::Region => key(raw),
        _ => None,
    }
}

/// What the bars make of the event, and the frame that owes when the view moved.
///
/// **The helper is asked once for both bars** (D3): the wheel over the region, a press on a track
/// and a drag that follows the pointer are one call, and the answer already names the axis it spoke
/// for. **A press is the bar's only where the bar is drawn**: a track the content collapsed answers
/// nothing, so the interior's press reaches [`click`] as it always did.
///
/// **Only a view that moved owes a frame** (rules 3 and 4): a turn the content refuses and a press
/// where the view already stands leave the screen as it was, and the event is not claimed.
fn scroll(raw: &Event, state: &mut DiagramState) -> Option<Action> {
    let area = state.region();
    let answer = state.scroll_area(area).handle(raw, MouseOnly);

    state.scrolled(answer).then_some(Action::ScreenChanged)
}

/// What a key means to the region.
///
/// **The four arrows displace the selected figure one cell** (rule 1 of #179): the action carries
/// the delta the arrow names and no identity, and the selection is read where the action is carried
/// out. A key that is no arrow is not the region's.
fn key(raw: &Event) -> Option<Action> {
    let (dx, dy) = displacement(raw)?;

    Some(Action::Move(dx, dy))
}

/// The delta the four arrows name, or `None` for any other event.
///
/// **A press with no modifiers and nothing else** (rule 8): a release and a repeat are not a press,
/// and a modified arrow is another combination. Right is one cell along the x axis, Down along the
/// y one, and the two negatives are the other directions.
fn displacement(event: &Event) -> Option<(i16, i16)> {
    let Event::Key(KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        ..
    }) = event
    else {
        return None;
    };

    match code {
        KeyCode::Right => Some((1, 0)),
        KeyCode::Left => Some((-1, 0)),
        KeyCode::Up => Some((0, -1)),
        KeyCode::Down => Some((0, 1)),
        _ => None,
    }
}

/// What a click means to the region.
///
/// **A left press inside the interior selects the shape the drawing records at the cell under it,
/// and a press on a cell nobody owns drops the selection** (rules 1 and 2). The offset asked for is
/// the click's own, counted from the interior's corner (see [`DiagramState::offset_of`]).
///
/// **Everything else is not the region's**, and it is answered with nothing so that the event goes
/// on to the next handler (rule 3): a press on the bar, the region's border or the status bar, a
/// press on a rectangle no frame drew into, and a press of any button but the left. A release is not
/// a press, so it is not the region's either.
///
/// **The answer is `ScreenChanged` only when the selection moved.** Selecting what is already
/// selected, or dropping a selection that is not there, changed nothing and owes no frame.
fn click(mouse: MouseEvent, state: &mut DiagramState) -> Option<Action> {
    if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
        return None;
    }

    let at = Position {
        x: mouse.column,
        y: mouse.row,
    };
    let owner = state.owner(state.offset_of(at)?);

    state.select(owner).then_some(Action::ScreenChanged)
}

/// Copies the drawing into the frame, one cell at a time.
///
/// **Only the cells the drawing holds are written.** The frame clears the whole screen before it
/// draws, so a cell the diagram does not speak for keeps the space the clear painted.
///
/// **The inversion is decided while the cell is copied** (D1): the cell's recorded owner is
/// compared with the selected identity, and the copy paints reverse video where the two agree. A
/// style on the core's cell would be a front end's decision in the domain, and a picture built for
/// the inversion would be one nobody clicked.
///
/// **A selected shape is not a second drawing.** The cells that are inverted are the ones the copy
/// writes and the record says belong to the selection, so a shape clipped by the window inverts the
/// part the window holds, and a shape behind another everywhere it draws inverts nothing (rule 5).
///
/// **`monospace_core::render` is deliberately not used.** It hands back the picture as a `String`,
/// and the buffer behind it is what answers a click; one drawing with one window is what keeps
/// those two the same thing.
fn draw_diagram(area: Rect, buf: &mut Buffer, state: &DiagramState, catalog: &GlyphCatalog) {
    if area.is_empty() {
        return;
    }

    for y in 0..area.height {
        for x in 0..area.width {
            let at = Offset { x, y };
            let Some(glyph) = state.glyph(at, catalog) else {
                continue;
            };

            let cell = &mut buf[(area.x + x, area.y + y)];
            cell.set_symbol(glyph);

            if state.is_cell_selected(at) {
                cell.set_style(Style::new().reversed());
            }
        }
    }
}

/// Paints the region the diagram goes into.
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

/// Lays the two bars on the region's border.
///
/// **On the border and not inside it** (D4): the vertical bar is drawn on the block's right column
/// and the horizontal one on its bottom row, over the glyphs the block put there and between its
/// four corners — which are the margin, so the run is the border's own from one corner to the next.
/// The window keeps every cell it had, so a bar that appears moves no bound. **Collapsed and not
/// minimized**: an axis whose content fits shows no bar at all, which is what leaves the empty
/// screen at rest a plain border.
///
/// **Each bar is drawn into its own run rather than through `ScrollArea`**: the helper lays its bars
/// inside an area it shrinks by one for every bar it shows, which is what a block of its own needs
/// and not what a border of ours does — through it, the track stopped a cell short of the far corner
/// and left a border glyph between its arrow and the corner. Drawing into the run keeps the two ends
/// of the track symmetric.
///
/// **The bar is the helper's own**, style and glyphs included: a `║` track with `▲` and `▼` at the
/// ends and a solid thumb, and `═`, `◄`, `►` across.
fn draw_bars(area: Rect, buf: &mut Buffer, state: &mut DiagramState) {
    if area.height < 2 || area.width < 2 {
        return;
    }

    let across = Scroll::horizontal().policy(ScrollbarPolicy::Collapse);
    let down = Scroll::vertical().policy(ScrollbarPolicy::Collapse);
    let (hscroll, vscroll) = state.scroll_states();

    across.render(
        Rect::new(area.x + 1, area.bottom() - 1, area.width - 2, 1),
        buf,
        hscroll,
    );
    down.render(
        Rect::new(area.right() - 1, area.y + 1, 1, area.height - 2),
        buf,
        vscroll,
    );
}

fn inner_region(area: Rect) -> Rect {
    let Rect {
        x,
        y,
        width,
        height,
    } = area;
    Rect {
        x: x + 1,
        y: y + 1,
        width: width.saturating_sub(2),
        height: height.saturating_sub(2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use ratatui_crossterm::crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers, MouseEvent};

    /// The region these tests draw into. **Its interior is `18×6` at `(1, 1)` of the screen**, which
    /// is what says where a click's column and row land.
    const REGION: Rect = Rect::new(0, 0, 20, 8);

    /// A description whose only shape is a light 4×3 box at `(2, 1)` of the window.
    const A_BOX: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 },
                      "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
    }"#;

    /// A description whose only shape is a light 30×9 box at `(0, 0)`: larger than the region's
    /// `18×6` interior on both axes, so both bars appear and the view has room to travel.
    const A_BOX_TOO_BIG: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                      "size": { "width": 30, "height": 9 }, "stroke": "light" } ]
    }"#;

    /// The state of a region that has not drawn a frame yet, over the diagram `json` describes.
    fn a_state_over(json: &str) -> DiagramState {
        let diagram = monospace_description::parse(json, &monospace_terminal_styles::all())
            .expect("the test's description is readable");

        DiagramState::new(monospace_editing::Session::new(diagram))
    }

    /// Paints the region into a frame of its own.
    fn paint(state: &mut DiagramState) -> Buffer {
        let mut buffer = Buffer::empty(REGION);
        draw(
            REGION,
            &mut buffer,
            state,
            &monospace_glyph_sets::all(),
            &Config::default().theme,
        );
        buffer
    }

    /// The event the terminal reports for a press of `button` at the screen position
    /// `(column, row)`.
    fn press(button: MouseButton, column: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(button),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// The event the terminal reports for a left press on the screen position `(column, row)`.
    fn click(column: u16, row: u16) -> Event {
        press(MouseButton::Left, column, row)
    }

    /// The screen position of the window's `(x, y)`: the interior's corner is the region's plus one
    /// in each direction.
    fn screen(x: u16, y: u16) -> Position {
        Position { x: x + 1, y: y + 1 }
    }

    /// **The bar sits on the region's border and takes no cell from the window** (rule 4). The
    /// window is the interior it always was, `18×6` at `(1, 1)`, and the bar is drawn over the
    /// block's own column and row: the last column of the window is still the window's, and the one
    /// after it is the bar's.
    ///
    /// **The run is the border's own from corner to corner** — rows `1` to `6` of the right column
    /// and columns `1` to `18` of the bottom row — so the arrows sit against the corners and no
    /// glyph of the block's is left between an arrow and one.
    #[test]
    fn the_bar_is_on_the_border_and_takes_no_cell_from_the_window() {
        let mut state = a_state_over(A_BOX_TOO_BIG);
        let buffer = paint(&mut state);

        assert_eq!(
            buffer[(19, 1)].symbol(),
            "▲",
            "the vertical bar does not begin on the right border"
        );
        assert_eq!(
            buffer[(19, 6)].symbol(),
            "▼",
            "the vertical bar does not end on the right border"
        );
        assert_eq!(
            buffer[(1, 7)].symbol(),
            "◄",
            "the horizontal bar does not begin on the bottom border"
        );
        assert_eq!(
            buffer[(18, 7)].symbol(),
            "►",
            "the horizontal bar does not end on the bottom border"
        );

        // The window's own last column and row are the window's: a press there is a press on the
        // drawing, and the bar's column and row are outside it.
        assert!(
            state.offset_of(screen(17, 5)).is_some(),
            "the bar took the window's last column"
        );
        assert_eq!(
            state.offset_of(screen(18, 5)),
            None,
            "the bar's column answered as the window"
        );
    }

    /// **A bar shows only where the content does not fit** (rule 4): a box inside the first screen
    /// leaves the region's border exactly as the block drew it, on both axes.
    #[test]
    fn the_bar_is_hidden_when_the_content_fits() {
        let mut state = a_state_over(A_BOX);
        let buffer = paint(&mut state);

        assert_eq!(
            buffer[(19, 1)].symbol(),
            "│",
            "the empty right border carries a bar"
        );
        assert_eq!(
            buffer[(1, 7)].symbol(),
            "─",
            "the empty bottom border carries a bar"
        );
    }

    /// **The four corners stay the block's** (rule 4): the bars are laid between them with a cell of
    /// margin at each end, so a scrolled drawing's border reads as a border drawn twice rather than
    /// as a bar that took the corners.
    #[test]
    fn the_corners_are_the_blocks() {
        let mut state = a_state_over(A_BOX_TOO_BIG);
        let buffer = paint(&mut state);

        for (at, glyph) in [((0, 0), "┌"), ((19, 0), "┐"), ((0, 7), "└"), ((19, 7), "┘")] {
            assert_eq!(
                buffer[at].symbol(),
                glyph,
                "the corner at {at:?} is not the block's"
            );
        }
    }

    /// **The window is one cell in from the region's corner, and a click is a position of the
    /// screen.** The box is at `(2, 1)` of the window, so the screen's `(3, 2)` asks about it, and
    /// the sum the terminal reports is the one that goes in — nothing is added to it on the way.
    #[test]
    fn the_click_offset_is_counted_from_the_interiors_corner() {
        let mut state = a_state_over(A_BOX);
        let _ = paint(&mut state);

        assert_eq!(
            state.offset_of(screen(2, 1)),
            Some(Offset { x: 2, y: 1 }),
            "the click's own position is not the offset asked for"
        );

        // The region's own row and column are the border's, and a press on either is outside the
        // window. A translation that forgot the interior's corner would answer `Some` here.
        assert_eq!(
            state.offset_of(Position { x: 0, y: 0 }),
            None,
            "a press on the region's corner reached the window"
        );
        assert_eq!(
            state.offset_of(Position { x: 3, y: 0 }),
            None,
            "a press on the region's top border reached the window"
        );
        assert_eq!(
            state.offset_of(Position { x: 0, y: 2 }),
            None,
            "a press on the region's left border reached the window"
        );
    }

    /// **A press of another button is not the region's**, and neither is a release of the left one:
    /// the notification a terminal sends when a button comes back up would otherwise select or drop
    /// a selection on every click.
    #[test]
    fn a_press_of_another_button_changes_nothing() {
        let mut state = a_state_over(A_BOX);
        let _ = paint(&mut state);
        let at = screen(2, 1);
        let _ = handle(&click(at.x, at.y), &mut state, Focus::Region);
        assert!(state.selected().is_some(), "the box was not selected");

        for button in [MouseButton::Right, MouseButton::Middle] {
            assert_eq!(
                handle(&press(button, at.x, at.y), &mut state, Focus::Region),
                None,
                "{button:?} was the region's"
            );
        }

        let released = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: at.x,
            row: at.y,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(
            handle(&released, &mut state, Focus::Region),
            None,
            "a release was a press"
        );

        assert!(
            state.selected().is_some(),
            "a press that is not the left button dropped the selection"
        );
    }

    /// The event the terminal reports for a press of `code` with no modifiers.
    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    /// **The four arrows are the region's while it holds the keyboard** (rule 6), each carrying the
    /// delta of its own axis and no identity (rule 1).
    #[test]
    fn the_arrows_are_the_regions_while_it_holds_the_keyboard() {
        let mut state = a_state_over(A_BOX);

        for (arrow, delta) in [
            (KeyCode::Right, (1, 0)),
            (KeyCode::Left, (-1, 0)),
            (KeyCode::Up, (0, -1)),
            (KeyCode::Down, (0, 1)),
        ] {
            assert_eq!(
                handle(&key(arrow), &mut state, Focus::Region),
                Some(Action::Move(delta.0, delta.1)),
                "{arrow:?} did not answer the movement"
            );
        }
    }

    /// **The action is the movement and not a figure** (rule 4): the region answers `Move` with
    /// nothing selected as well, and the handler neither selects nor performs anything.
    #[test]
    fn the_arrows_say_only_that_they_move() {
        let mut state = a_state_over(A_BOX);
        assert_eq!(state.selected(), None, "the test starts with a selection");

        let answer = handle(&key(KeyCode::Right), &mut state, Focus::Region);

        assert_eq!(answer, Some(Action::Move(1, 0)));
        assert_eq!(
            state.selected(),
            None,
            "the region minted a selection into the action"
        );
    }

    /// **A move is the arrow and no modifier** (rule 8): another combination, a release and a
    /// repeat are not the region's, and neither is a key that is no arrow.
    #[test]
    fn a_move_is_the_arrow_and_no_modifier() {
        let mut state = a_state_over(A_BOX);

        for modifiers in [
            KeyModifiers::SHIFT,
            KeyModifiers::CONTROL,
            KeyModifiers::ALT,
        ] {
            assert_eq!(
                handle(
                    &Event::Key(KeyEvent::new(KeyCode::Right, modifiers)),
                    &mut state,
                    Focus::Region,
                ),
                None,
                "a modified arrow was the region's"
            );
        }

        for kind in [KeyEventKind::Release, KeyEventKind::Repeat] {
            let event = Event::Key(KeyEvent::new_with_kind(
                KeyCode::Right,
                KeyModifiers::NONE,
                kind,
            ));
            assert_eq!(
                handle(&event, &mut state, Focus::Region),
                None,
                "{kind:?} was a press"
            );
        }

        assert_eq!(
            handle(&key(KeyCode::Char('x')), &mut state, Focus::Region),
            None,
            "a key that is no arrow was the region's"
        );
    }

    /// **A bar that holds the keyboard answers the arrows itself** (rule 6): `Up` and `Down` are
    /// nobody's there, so the region must not move the figure from a leaf that does not hold it.
    #[test]
    fn an_arrow_with_the_bar_focused_moves_nothing() {
        let mut state = a_state_over(A_BOX);
        let _ = paint(&mut state);
        let _ = handle(&click(3, 2), &mut state, Focus::Region);
        assert!(state.selected().is_some(), "the box was not selected");

        for arrow in [KeyCode::Up, KeyCode::Down] {
            assert_eq!(
                handle(&key(arrow), &mut state, Focus::Bar),
                None,
                "{arrow:?} reached the region while the bar held the keyboard"
            );
        }

        assert!(state.selected().is_some(), "the box lost its selection");
    }
}
