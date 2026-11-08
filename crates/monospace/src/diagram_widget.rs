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
    layout::Rect,
    style::Style,
    widgets::{StatefulWidget, Widget},
};
use ratatui_crossterm::crossterm::event::Event;

use crate::chrome::{Focus, Target};
use crate::diagram_state::DiagramState;
use crate::vocabulary::Reply;

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
/// **The region answers a click and the keys the contextual table offers.** A click is the
/// region's whatever leaf holds the keyboard (D1), and a key is the region's only while it does.
/// [`crate::context_table::dispatch_key`] is what the region does with the keyboard it is given,
/// and a key the table does not offer is `Reply::Ignored` so that the next handler is asked.
///
/// **Everything else is answered with nothing** so that the event goes on to the next handler
/// (rule 3).
pub(crate) fn handle(raw: &Event, state: &mut DiagramState, focus: Focus, target: Target) -> Reply {
    match scroll(raw, state) {
        Reply::Ignored => {}
        reply => return reply,
    }

    match raw {
        Event::Mouse(_) => click(target, state),
        Event::Key(key) if focus == Focus::Region => crate::context_table::dispatch_key(state, key),
        _ => Reply::Ignored,
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
/// where the view already stands leave the screen as it was, but the bar took the event, so the
/// answer is `Consumed` rather than `Ignored` and the region's click does not see it twice.
fn scroll(raw: &Event, state: &mut DiagramState) -> Reply {
    let area = state.region();
    let answer = state.scroll_area(area).handle(raw, MouseOnly);

    state.scrolled(answer)
}

/// What a click means to the region.
///
/// **A left press inside the interior selects the shape the drawing records at the cell the router
/// found, and a press on a cell nobody owns drops the selection** (rules 1 and 2). The cell is
/// [`Target::Interior`]'s, counted from the interior's corner, so the region never reads the
/// pointer a second time.
///
/// **Everything else is not the region's**, and it is answered with nothing so that the event goes
/// on to the next handler (rule 3): a press on the bar, the region's border or the status bar, and
/// a press on a rectangle no frame drew into. A press of another button is no target at all, and a
/// release is not a press, so neither is the region's either.
///
/// **The answer is `Repaint` only when the selection moved.** Selecting what is already selected,
/// or dropping a selection that is not there, changed nothing and owes no frame — but the press was
/// still the region's, so it is `Consumed` rather than `Ignored`.
fn click(target: Target, state: &mut DiagramState) -> Reply {
    let Target::Interior(offset) = target else {
        return Reply::Ignored;
    };

    let owner = state.owner(offset);

    if state.select(owner) {
        Reply::Repaint
    } else {
        Reply::Consumed
    }
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
    use crate::diagram_state::{End, SubMode};
    use crate::vocabulary::Action;
    use monospace_core::ShapeId;
    use monospace_diagram::Delta;
    use monospace_editing::Command;
    use ratatui_core::layout::Position;
    use ratatui_crossterm::crossterm::event::{
        KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };
    use std::num::NonZeroU16;

    /// The region these tests draw into. **Its interior is `18×6` at `(1, 1)` of the screen**, which
    /// is what says where a click's column and row land.
    const REGION: Rect = Rect::new(0, 0, 20, 8);

    /// A description whose only shape is a light 4×3 box at `(2, 1)` of the window.
    const A_BOX: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 },
                      "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
    }"#;

    /// A description whose only shape is a light horizontal line of length 6 at `(2, 1)`.
    const A_LINE: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "line", "id": 1, "at": { "x": 2, "y": 1 },
                      "len": 6, "orientation": "horizontal", "stroke": "light" } ]
    }"#;

    /// A description whose only shape is a light vertical line of length 3 at `(2, 1)`.
    const A_VERTICAL_LINE: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "line", "id": 1, "at": { "x": 2, "y": 1 },
                      "len": 3, "orientation": "vertical", "stroke": "light" } ]
    }"#;

    /// A description whose only shape is a light connector between `(2, 1)` and `(10, 1)`.
    const A_CONNECTOR: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "connector", "id": 1,
                      "from": { "at": { "kind": "point", "x": 2, "y": 1 },
                                "leaving": "right", "terminal": { "kind": "arm" } },
                      "to": { "at": { "kind": "point", "x": 10, "y": 1 },
                              "leaving": "left", "terminal": { "kind": "arm" } },
                      "stroke": "light" } ]
    }"#;

    /// A description with two boxes and a connector attached between them.
    const AN_ATTACHED_CONNECTOR: &str = r#"{
        "next_id": 4,
        "shapes": [
            { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 4 }, "stroke": "light" },
            { "kind": "box", "id": 2, "at": { "x": 20, "y": 0 }, "size": { "width": 8, "height": 4 }, "stroke": "light" },
            { "kind": "connector", "id": 3,
              "from": { "at": { "kind": "reference", "shape": 1, "anchor": "top", "offset": { "dx": 0, "dy": 0 }, "out": 0 },
                        "terminal": { "kind": "arm" } },
              "to":   { "at": { "kind": "reference", "shape": 2, "anchor": "left", "offset": { "dx": 0, "dy": 1 }, "out": 2 },
                        "terminal": { "kind": "symbol", "style": "solid" } },
              "stroke": "light" }
        ]
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
        let diagram =
            monospace_description::parse(json).expect("the test's description is readable");

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

    /// Hands the event to the region the way `callbacks` does: the target of a left press is read
    /// from the frame, and every other event carries none.
    fn handle(event: &Event, state: &mut DiagramState, focus: Focus) -> Reply {
        let target = match event {
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                state.target_at(Position::new(mouse.column, mouse.row))
            }
            _ => Target::Nothing,
        };

        super::handle(event, state, focus, target)
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
                Reply::Ignored,
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
            Reply::Ignored,
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
        let id = ShapeId::new(NonZeroU16::new(1).unwrap());
        state.select(Some(id));

        for (arrow, delta) in [
            (KeyCode::Right, (1, 0)),
            (KeyCode::Left, (-1, 0)),
            (KeyCode::Up, (0, -1)),
            (KeyCode::Down, (0, 1)),
        ] {
            assert_eq!(
                handle(&key(arrow), &mut state, Focus::Region),
                Reply::Act(Action::Command(Command::Move {
                    id,
                    by: (delta.0, delta.1).into(),
                })),
                "{arrow:?} did not answer the movement"
            );
        }
    }

    /// **The action is the command naming the selected figure**: the region answers `Command::Move` with
    /// the delta the arrow names and the selected shape's identity.
    #[test]
    fn the_arrows_say_only_that_they_move() {
        let mut state = a_state_over(A_BOX);
        let id = ShapeId::new(NonZeroU16::new(1).unwrap());
        state.select(Some(id));

        let answer = handle(&key(KeyCode::Right), &mut state, Focus::Region);

        assert_eq!(
            answer,
            Reply::Act(Action::Command(Command::Move {
                id,
                by: (1, 0).into(),
            }))
        );
        assert_eq!(
            state.selected(),
            Some(id),
            "the region changed the selection"
        );
    }

    /// **The arrows are ignored when nothing is selected**: the region answers `Reply::Ignored` so
    /// the key is left unhandled when there is no figure to move.
    #[test]
    fn arrows_are_ignored_when_nothing_is_selected() {
        let mut state = a_state_over(A_BOX);
        assert_eq!(state.selected(), None);

        for arrow in [KeyCode::Right, KeyCode::Left, KeyCode::Up, KeyCode::Down] {
            assert_eq!(
                handle(&key(arrow), &mut state, Focus::Region),
                Reply::Ignored,
                "{arrow:?} was not ignored with nothing selected"
            );
        }
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
                Reply::Ignored,
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
                Reply::Ignored,
                "{kind:?} was a press"
            );
        }

        assert_eq!(
            handle(&key(KeyCode::Char('x')), &mut state, Focus::Region),
            Reply::Ignored,
            "a key that is no arrow was the region's"
        );
    }

    /// **Shift plus an arrow resizes when a box is selected** (rule 1 of #284).
    #[test]
    fn shift_arrows_resize_when_a_box_is_selected() {
        let mut state = a_state_over(A_BOX);
        let _ = paint(&mut state);
        let at = screen(2, 1);
        let _ = handle(&click(at.x, at.y), &mut state, Focus::Region);
        let id = state.selected().expect("box is selected");

        for (arrow, delta) in [
            (KeyCode::Right, (1, 0)),
            (KeyCode::Left, (-1, 0)),
            (KeyCode::Up, (0, -1)),
            (KeyCode::Down, (0, 1)),
        ] {
            assert_eq!(
                handle(
                    &Event::Key(KeyEvent::new(arrow, KeyModifiers::SHIFT)),
                    &mut state,
                    Focus::Region,
                ),
                Reply::Act(Action::Command(Command::Resize {
                    id,
                    by: (delta.0, delta.1).into(),
                })),
                "{arrow:?} with Shift did not answer the resize"
            );
        }
    }

    /// **Shift plus horizontal arrows resize a horizontal line; vertical arrows are ignored** (D1 of #301).
    #[test]
    fn shift_arrows_resize_a_horizontal_line_along_its_row() {
        let mut state = a_state_over(A_LINE);
        let id = ShapeId::new(NonZeroU16::new(1).unwrap());
        state.select(Some(id));

        assert_eq!(
            handle(
                &Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT)),
                &mut state,
                Focus::Region,
            ),
            Reply::Act(Action::Command(Command::Resize {
                id,
                by: (-1, 0).into(),
            }))
        );
        assert_eq!(
            handle(
                &Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT)),
                &mut state,
                Focus::Region,
            ),
            Reply::Act(Action::Command(Command::Resize {
                id,
                by: (1, 0).into(),
            }))
        );
        assert_eq!(
            handle(
                &Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT)),
                &mut state,
                Focus::Region,
            ),
            Reply::Ignored
        );
        assert_eq!(
            handle(
                &Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT)),
                &mut state,
                Focus::Region,
            ),
            Reply::Ignored
        );
    }

    /// **Shift plus vertical arrows resize a vertical line; horizontal arrows are ignored** (D1 of #301).
    #[test]
    fn shift_arrows_resize_a_vertical_line_along_its_column() {
        let mut state = a_state_over(A_VERTICAL_LINE);
        let id = ShapeId::new(NonZeroU16::new(1).unwrap());
        state.select(Some(id));

        assert_eq!(
            handle(
                &Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT)),
                &mut state,
                Focus::Region,
            ),
            Reply::Act(Action::Command(Command::Resize {
                id,
                by: (0, -1).into(),
            }))
        );
        assert_eq!(
            handle(
                &Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT)),
                &mut state,
                Focus::Region,
            ),
            Reply::Act(Action::Command(Command::Resize {
                id,
                by: (0, 1).into(),
            }))
        );
        assert_eq!(
            handle(
                &Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT)),
                &mut state,
                Focus::Region,
            ),
            Reply::Ignored
        );
        assert_eq!(
            handle(
                &Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT)),
                &mut state,
                Focus::Region,
            ),
            Reply::Ignored
        );
    }

    /// **Shift plus an arrow is ignored when a connector is selected** (rule 5 of contextual bar).
    #[test]
    fn shift_arrows_are_ignored_when_a_connector_is_selected() {
        let mut state = a_state_over(A_CONNECTOR);
        state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));
        assert_eq!(
            handle(
                &Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT)),
                &mut state,
                Focus::Region,
            ),
            Reply::Ignored
        );
    }

    /// **Shift plus an arrow is ignored when nothing is selected** (rule 4 of #284).
    #[test]
    fn shift_arrows_are_ignored_when_nothing_is_selected() {
        let mut state = a_state_over(A_BOX);
        assert_eq!(state.selected(), None);

        assert_eq!(
            handle(
                &Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT)),
                &mut state,
                Focus::Region,
            ),
            Reply::Ignored
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
                Reply::Ignored,
                "{arrow:?} reached the region while the bar held the keyboard"
            );
        }

        assert!(state.selected().is_some(), "the box lost its selection");
    }

    /// **Escape clears the active selection** (D1 of #279): pressing Escape while the region holds
    /// the keyboard drops the active selection and requests a repaint.
    #[test]
    fn pressing_esc_in_the_region_clears_the_selection_and_requests_repaint() {
        let mut state = a_state_over(A_BOX);
        let _ = paint(&mut state);
        let _ = handle(&click(3, 2), &mut state, Focus::Region);
        assert!(state.selected().is_some(), "the box was not selected");

        let outcome = handle(&key(KeyCode::Esc), &mut state, Focus::Region);

        assert_eq!(outcome, Reply::Repaint);
        assert_eq!(state.selected(), None);
    }

    /// **Escape with nothing selected changes nothing and is ignored** (D1 of #279): no event is
    /// answered and no frame is owed.
    #[test]
    fn pressing_esc_with_nothing_selected_is_ignored() {
        let mut state = a_state_over(A_BOX);
        assert_eq!(state.selected(), None);

        let outcome = handle(&key(KeyCode::Esc), &mut state, Focus::Region);

        assert_eq!(outcome, Reply::Ignored);
        assert_eq!(state.selected(), None);
    }

    /// **Escape while the bar holds the keyboard does not drop the selection** (D1 of #279): the key
    /// is not the region's while the bar holds the keyboard.
    #[test]
    fn escape_with_the_bar_focused_does_not_drop_the_selection() {
        let mut state = a_state_over(A_BOX);
        let _ = paint(&mut state);
        let _ = handle(&click(3, 2), &mut state, Focus::Region);
        assert!(state.selected().is_some(), "the box was not selected");

        let outcome = handle(&key(KeyCode::Esc), &mut state, Focus::Bar);

        assert_eq!(outcome, Reply::Ignored);
        assert!(state.selected().is_some(), "the box lost its selection");
    }

    /// Keys the region must never answer, whatever the context: another change's shortcuts,
    /// the bar's own (`Tab`), and keys nobody claims.
    const NEVER_AN_ACTION: [(KeyCode, KeyModifiers); 6] = [
        (KeyCode::Char('x'), KeyModifiers::NONE),
        (KeyCode::Char('a'), KeyModifiers::NONE),
        (KeyCode::Enter, KeyModifiers::NONE),
        (KeyCode::Tab, KeyModifiers::NONE),
        (KeyCode::BackTab, KeyModifiers::SHIFT),
        (KeyCode::F(1), KeyModifiers::NONE),
    ];

    const ALL_POSSIBLE_CONTEXT_KEYS: [(KeyCode, KeyModifiers); 27] = [
        (KeyCode::Right, KeyModifiers::NONE),
        (KeyCode::Left, KeyModifiers::NONE),
        (KeyCode::Up, KeyModifiers::NONE),
        (KeyCode::Down, KeyModifiers::NONE),
        (KeyCode::Right, KeyModifiers::SHIFT),
        (KeyCode::Left, KeyModifiers::SHIFT),
        (KeyCode::Up, KeyModifiers::SHIFT),
        (KeyCode::Down, KeyModifiers::SHIFT),
        (KeyCode::Home, KeyModifiers::NONE),
        (KeyCode::End, KeyModifiers::NONE),
        (KeyCode::PageUp, KeyModifiers::NONE),
        (KeyCode::PageDown, KeyModifiers::NONE),
        (KeyCode::Char('t'), KeyModifiers::NONE),
        (KeyCode::Char('i'), KeyModifiers::NONE),
        (KeyCode::Char('s'), KeyModifiers::NONE),
        (KeyCode::Char('z'), KeyModifiers::NONE),
        (KeyCode::Char('r'), KeyModifiers::NONE),
        (KeyCode::Char('+'), KeyModifiers::NONE),
        (KeyCode::Char('+'), KeyModifiers::SHIFT),
        (KeyCode::Char('-'), KeyModifiers::NONE),
        (KeyCode::Char('1'), KeyModifiers::NONE),
        (KeyCode::Char('2'), KeyModifiers::NONE),
        (KeyCode::Char('3'), KeyModifiers::NONE),
        (KeyCode::Char('4'), KeyModifiers::NONE),
        (KeyCode::Char('5'), KeyModifiers::NONE),
        (KeyCode::Delete, KeyModifiers::NONE),
        (KeyCode::Esc, KeyModifiers::NONE),
    ];

    /// **The rule holds three ways**: the table the test writes, the bar the application draws and
    /// the keys the region answers all say the same thing.
    ///
    /// **An unoffered key is answered with `Reply::Ignored`**, so it falls through to whatever
    /// comes after the region; a key the table offers is never ignored, because the bar said it
    /// was there.
    fn assert_offer_matches_acceptance(
        fixture: &str,
        selected: Option<u16>,
        sub_mode: Option<crate::diagram_state::SubMode>,
    ) {
        let make_state = || {
            let mut state = a_state_over(fixture);
            if let Some(id) = selected {
                state.select(Some(ShapeId::new(NonZeroU16::new(id).unwrap())));
            }
            if let Some(mode) = sub_mode {
                state.set_sub_mode(Some(mode));
            }
            state
        };

        let state = make_state();
        let bar = crate::status::command_line(&state);

        let mut offered_keys = Vec::new();
        for block in crate::context_table::context_blocks(&state) {
            if let Some(category) = block.category {
                assert!(
                    bar.contains(category),
                    "bar missing category {category} in: {bar}"
                );
            }
            for action in block.actions {
                assert!(
                    bar.contains(action.display),
                    "bar missing action display {:?} in: {bar}",
                    action.display
                );
                for (code, modifiers) in action.pattern.samples() {
                    offered_keys.push((code, modifiers));
                    let mut s = make_state();
                    let event = Event::Key(KeyEvent::new(code, modifiers));
                    let reply = handle(&event, &mut s, Focus::Region);
                    assert_ne!(
                        reply,
                        Reply::Ignored,
                        "offered key {code:?} with modifiers {modifiers:?} in context with bar {bar:?} was ignored"
                    );
                }
            }
        }

        for (code, modifiers) in ALL_POSSIBLE_CONTEXT_KEYS {
            if !offered_keys.contains(&(code, modifiers)) {
                let mut s = make_state();
                let event = Event::Key(KeyEvent::new(code, modifiers));
                let reply = handle(&event, &mut s, Focus::Region);
                assert_eq!(
                    reply,
                    Reply::Ignored,
                    "unoffered key {code:?} with modifiers {modifiers:?} in context with bar {bar:?} answered unexpected reply: {reply:?}"
                );
            }
        }

        for (code, modifiers) in NEVER_AN_ACTION {
            let mut s = make_state();
            let event = Event::Key(KeyEvent::new(code, modifiers));
            let reply = handle(&event, &mut s, Focus::Region);

            assert_eq!(
                reply,
                Reply::Ignored,
                "{code:?} with modifiers {modifiers:?} was unexpected in: {bar}"
            );
        }

        for event in crate::shortcuts::events_unclaimed() {
            if let Event::Key(k) = event
                && offered_keys.contains(&(k.code, k.modifiers))
            {
                continue;
            }
            let mut s = make_state();
            let reply = handle(&event, &mut s, Focus::Region);
            assert_eq!(
                reply,
                Reply::Ignored,
                "global unclaimed shortcut {event:?} was NOT ignored in: {bar}"
            );
        }
    }

    /// **In IDLE, key acceptance matches what the contextual bar offers**: the bar offers only
    /// selection via click, so all contextual keys (arrows, Shift+arrows, Esc) are ignored.
    #[test]
    fn key_acceptance_matches_contextual_bar_in_idle() {
        assert_offer_matches_acceptance(A_BOX, None, None);
    }

    /// **For a selected box, key acceptance matches what the contextual bar offers**: the bar offers
    /// MOVE, RESIZE, and Deselect, so plain arrows, Shift+arrows, and Esc are all accepted.
    #[test]
    fn key_acceptance_matches_contextual_bar_for_selected_box() {
        assert_offer_matches_acceptance(A_BOX, Some(1), None);
    }

    /// **For a selected horizontal line, key acceptance matches what the contextual bar offers**:
    /// the bar offers MOVE, RESIZE: Shift+←→, and Deselect; horizontal Shift+arrows are accepted,
    /// and vertical Shift+arrows are ignored.
    #[test]
    fn key_acceptance_matches_contextual_bar_for_selected_horizontal_line() {
        assert_offer_matches_acceptance(A_LINE, Some(1), None);
    }

    /// **For a selected vertical line, key acceptance matches what the contextual bar offers**:
    /// the bar offers MOVE, RESIZE: Shift+↑↓, and Deselect; vertical Shift+arrows are accepted,
    /// and horizontal Shift+arrows are ignored.
    #[test]
    fn key_acceptance_matches_contextual_bar_for_selected_vertical_line() {
        assert_offer_matches_acceptance(A_VERTICAL_LINE, Some(1), None);
    }

    /// **For a selected connector, key acceptance matches what the contextual bar offers**: the bar
    /// offers MOVE and Deselect, but no RESIZE; plain arrows and Esc are accepted, and Shift+arrows
    /// are ignored.
    #[test]
    fn key_acceptance_matches_contextual_bar_for_selected_connector() {
        assert_offer_matches_acceptance(A_CONNECTOR, Some(1), None);
    }

    #[test]
    fn pressing_i_inverts_connector_terminals_when_selected() {
        let mut state = a_state_over(A_CONNECTOR);
        let id = ShapeId::new(NonZeroU16::new(1).unwrap());
        state.select(Some(id));

        let answer = handle(
            &Event::Key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE)),
            &mut state,
            Focus::Region,
        );
        assert_eq!(
            answer,
            Reply::Act(Action::Command(Command::InvertTerminals { id }))
        );

        // Pressing i when a box is selected is ignored.
        let mut box_state = a_state_over(A_BOX);
        box_state.select(Some(id));
        let box_answer = handle(
            &Event::Key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE)),
            &mut box_state,
            Focus::Region,
        );
        assert_eq!(box_answer, Reply::Ignored);

        // Pressing i when nothing is selected is ignored.
        let mut idle_state = a_state_over(A_CONNECTOR);
        let idle_answer = handle(
            &Event::Key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE)),
            &mut idle_state,
            Focus::Region,
        );
        assert_eq!(idle_answer, Reply::Ignored);
    }

    /// **In `STROKE_SELECT`, key acceptance matches what the contextual bar offers**: the bar offers
    /// digits 1..=5 and Esc Cancel; offered keys are accepted, and unoffered keys are ignored.
    #[test]
    fn key_acceptance_matches_contextual_bar_in_stroke_select() {
        assert_offer_matches_acceptance(
            A_BOX,
            Some(1),
            Some(crate::diagram_state::SubMode::StrokeSelect),
        );
    }

    /// **In `ORDER_MODE`, key acceptance matches what the contextual bar offers**: the bar offers
    /// `Home`, `End`, `PageUp`, `PageDown` and `Esc`; offered keys are accepted, and unoffered keys
    /// are ignored.
    #[test]
    fn key_acceptance_matches_contextual_bar_in_order_mode() {
        assert_offer_matches_acceptance(
            A_BOX,
            Some(1),
            Some(crate::diagram_state::SubMode::OrderMode),
        );
    }

    #[test]
    fn key_acceptance_matches_contextual_bar_in_endpoint_sub_mode() {
        assert_offer_matches_acceptance(A_CONNECTOR, Some(1), Some(SubMode::Endpoint(End::From)));
        assert_offer_matches_acceptance(A_CONNECTOR, Some(1), Some(SubMode::Endpoint(End::To)));
        assert_offer_matches_acceptance(
            AN_ATTACHED_CONNECTOR,
            Some(3),
            Some(SubMode::Endpoint(End::From)),
        );
        assert_offer_matches_acceptance(
            AN_ATTACHED_CONNECTOR,
            Some(3),
            Some(SubMode::Endpoint(End::To)),
        );
    }

    #[test]
    fn endpoint_sub_mode_moves_adjusts_out_and_resets_endpoints() {
        let mut state = a_state_over(AN_ATTACHED_CONNECTOR);
        let id = ShapeId::new(NonZeroU16::new(3).unwrap());
        state.select(Some(id));
        state.set_sub_mode(Some(SubMode::Endpoint(End::From)));

        // From is on Top anchor: horizontal arrows move dx, vertical arrows are ignored
        let right_arrow = key(KeyCode::Right);
        assert_eq!(
            handle(&right_arrow, &mut state, Focus::Region),
            Reply::Act(Action::Command(Command::MoveEndpoint {
                id,
                end: End::From,
                by: Delta { dx: 1, dy: 0 },
                out: 0,
            }))
        );
        let up_arrow = key(KeyCode::Up);
        assert_eq!(handle(&up_arrow, &mut state, Focus::Region), Reply::Ignored);

        // Adjust out on attached endpoint: '+' and '-'
        let key_plus = key(KeyCode::Char('+'));
        assert_eq!(
            handle(&key_plus, &mut state, Focus::Region),
            Reply::Act(Action::Command(Command::MoveEndpoint {
                id,
                end: End::From,
                by: Delta::ZERO,
                out: 1,
            }))
        );
        let key_minus = key(KeyCode::Char('-'));
        assert_eq!(
            handle(&key_minus, &mut state, Focus::Region),
            Reply::Act(Action::Command(Command::MoveEndpoint {
                id,
                end: End::From,
                by: Delta::ZERO,
                out: -1,
            }))
        );

        // Reset is not offered when offset is 0 and out is 0
        let key_r = key(KeyCode::Char('r'));
        assert_eq!(handle(&key_r, &mut state, Focus::Region), Reply::Ignored);

        // Switch to To endpoint (which has offset: (0, 1), out: 2 on Left anchor)
        state.set_sub_mode(Some(SubMode::Endpoint(End::To)));

        // Vertical arrows move dy, horizontal arrows are ignored
        let down_arrow = key(KeyCode::Down);
        assert_eq!(
            handle(&down_arrow, &mut state, Focus::Region),
            Reply::Act(Action::Command(Command::MoveEndpoint {
                id,
                end: End::To,
                by: Delta { dx: 0, dy: 1 },
                out: 0,
            }))
        );
        assert_eq!(
            handle(&right_arrow, &mut state, Focus::Region),
            Reply::Ignored
        );

        // Reset IS offered and accepted on To endpoint
        assert_eq!(
            handle(&key_r, &mut state, Focus::Region),
            Reply::Act(Action::Command(Command::ResetEndpoint { id, end: End::To }))
        );

        // On detached endpoint (A_CONNECTOR): all 4 arrows move, '+', '-', 'r' are ignored
        let mut detached_state = a_state_over(A_CONNECTOR);
        let conn_id = ShapeId::new(NonZeroU16::new(1).unwrap());
        detached_state.select(Some(conn_id));
        detached_state.set_sub_mode(Some(SubMode::Endpoint(End::From)));

        assert_eq!(
            handle(&up_arrow, &mut detached_state, Focus::Region),
            Reply::Act(Action::Command(Command::MoveEndpoint {
                id: conn_id,
                end: End::From,
                by: Delta { dx: 0, dy: -1 },
                out: 0,
            }))
        );
        assert_eq!(
            handle(&key_plus, &mut detached_state, Focus::Region),
            Reply::Ignored
        );
        assert_eq!(
            handle(&key_minus, &mut detached_state, Focus::Region),
            Reply::Ignored
        );
        assert_eq!(
            handle(&key_r, &mut detached_state, Focus::Region),
            Reply::Ignored
        );
    }

    #[test]
    fn pressing_1_and_2_enters_and_switches_endpoint_sub_mode_until_esc() {
        let mut state = a_state_over(A_CONNECTOR);
        let id = ShapeId::new(NonZeroU16::new(1).unwrap());
        state.select(Some(id));

        let key_1 = key(KeyCode::Char('1'));
        let key_2 = key(KeyCode::Char('2'));
        let key_esc = key(KeyCode::Esc);

        // Pressing '1' enters End::From and returns Reply::Repaint
        let answer = handle(&key_1, &mut state, Focus::Region);
        assert_eq!(answer, Reply::Repaint);
        assert_eq!(state.sub_mode(), Some(SubMode::Endpoint(End::From)));
        assert_eq!(state.selected(), Some(id));

        // Pressing '1' again while in End::From returns Reply::Ignored and stays in End::From
        let answer = handle(&key_1, &mut state, Focus::Region);
        assert_eq!(answer, Reply::Ignored);
        assert_eq!(state.sub_mode(), Some(SubMode::Endpoint(End::From)));

        // Pressing '2' while in End::From switches to End::To and returns Reply::Repaint
        let answer = handle(&key_2, &mut state, Focus::Region);
        assert_eq!(answer, Reply::Repaint);
        assert_eq!(state.sub_mode(), Some(SubMode::Endpoint(End::To)));
        assert_eq!(state.selected(), Some(id));

        // Pressing '2' again while in End::To returns Reply::Ignored and stays in End::To
        let answer = handle(&key_2, &mut state, Focus::Region);
        assert_eq!(answer, Reply::Ignored);
        assert_eq!(state.sub_mode(), Some(SubMode::Endpoint(End::To)));

        // Pressing '1' while in End::To switches back to End::From and returns Reply::Repaint
        let answer = handle(&key_1, &mut state, Focus::Region);
        assert_eq!(answer, Reply::Repaint);
        assert_eq!(state.sub_mode(), Some(SubMode::Endpoint(End::From)));
        assert_eq!(state.selected(), Some(id));

        // Pressing Esc leaves sub-mode (None), keeps ShapeId selected, and returns Reply::Repaint
        let answer = handle(&key_esc, &mut state, Focus::Region);
        assert_eq!(answer, Reply::Repaint);
        assert_eq!(state.sub_mode(), None);
        assert_eq!(state.selected(), Some(id));

        // Verify state.session().can_undo() is false (zero history steps recorded)
        assert!(!state.session().can_undo());

        // Pressing '1' or '2' on a selected box returns Reply::Ignored and leaves sub_mode() as None
        let mut box_state = a_state_over(A_BOX);
        box_state.select(Some(id));
        assert_eq!(
            handle(&key_1, &mut box_state, Focus::Region),
            Reply::Ignored
        );
        assert_eq!(box_state.sub_mode(), None);
        assert_eq!(
            handle(&key_2, &mut box_state, Focus::Region),
            Reply::Ignored
        );
        assert_eq!(box_state.sub_mode(), None);

        // Pressing '1' or '2' on a selected line returns Reply::Ignored and leaves sub_mode() as None
        let mut line_state = a_state_over(A_LINE);
        line_state.select(Some(id));
        assert_eq!(
            handle(&key_1, &mut line_state, Focus::Region),
            Reply::Ignored
        );
        assert_eq!(line_state.sub_mode(), None);
        assert_eq!(
            handle(&key_2, &mut line_state, Focus::Region),
            Reply::Ignored
        );
        assert_eq!(line_state.sub_mode(), None);

        // Pressing '1' or '2' in IDLE returns Reply::Ignored and leaves sub_mode() as None
        let mut idle_state = a_state_over(A_CONNECTOR);
        assert_eq!(
            handle(&key_1, &mut idle_state, Focus::Region),
            Reply::Ignored
        );
        assert_eq!(idle_state.sub_mode(), None);
        assert_eq!(
            handle(&key_2, &mut idle_state, Focus::Region),
            Reply::Ignored
        );
        assert_eq!(idle_state.sub_mode(), None);
    }

    #[test]
    fn arrows_are_ignored_when_the_selection_names_a_figure_the_drawing_does_not_record() {
        let mut state = a_state_over(A_BOX);
        state.select(Some(ShapeId::new(NonZeroU16::new(999).unwrap())));
        assert_eq!(state.selected_shape(), None);

        let answer = handle(
            &Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
            &mut state,
            Focus::Region,
        );
        assert_eq!(answer, Reply::Ignored);
    }

    #[test]
    fn pressing_s_on_selected_shape_enters_stroke_select_sub_mode() {
        let mut state = a_state_over(A_BOX);
        state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));

        let answer = handle(
            &Event::Key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE)),
            &mut state,
            Focus::Region,
        );
        assert_eq!(answer, Reply::Repaint);
        assert_eq!(
            state.sub_mode(),
            Some(crate::diagram_state::SubMode::StrokeSelect)
        );
    }

    #[test]
    fn pressing_s_in_idle_is_ignored() {
        let mut state = a_state_over(A_BOX);
        assert_eq!(state.selected(), None);

        let answer = handle(
            &Event::Key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE)),
            &mut state,
            Focus::Region,
        );
        assert_eq!(answer, Reply::Ignored);
        assert_eq!(state.sub_mode(), None);
    }

    #[test]
    fn digits_in_stroke_select_dispatch_set_stroke_and_clear_sub_mode() {
        use monospace_diagram::Stroke;

        let pairs = [
            ('1', "light"),
            ('2', "double"),
            ('3', "light-round"),
            ('4', "heavy"),
            ('5', "ascii"),
        ];

        for (c, stroke_name) in pairs {
            let mut state = a_state_over(A_BOX);
            let id = ShapeId::new(NonZeroU16::new(1).unwrap());
            state.select(Some(id));
            state.set_sub_mode(Some(crate::diagram_state::SubMode::StrokeSelect));

            let answer = handle(
                &Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)),
                &mut state,
                Focus::Region,
            );
            assert_eq!(
                answer,
                Reply::Act(Action::Command(Command::SetStroke {
                    id,
                    stroke: Stroke::from(stroke_name),
                }))
            );
            assert_eq!(state.sub_mode(), None);
        }
    }

    #[test]
    fn escape_in_stroke_select_cancels_sub_mode_and_leaves_figure_selected() {
        let mut state = a_state_over(A_BOX);
        let id = ShapeId::new(NonZeroU16::new(1).unwrap());
        state.select(Some(id));
        state.set_sub_mode(Some(crate::diagram_state::SubMode::StrokeSelect));

        let answer = handle(
            &Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            &mut state,
            Focus::Region,
        );
        assert_eq!(answer, Reply::Repaint);
        assert_eq!(state.sub_mode(), None);
        assert_eq!(state.selected(), Some(id));
    }

    #[test]
    fn unoffered_keys_in_stroke_select_are_ignored() {
        let mut state = a_state_over(A_BOX);
        state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));
        state.set_sub_mode(Some(crate::diagram_state::SubMode::StrokeSelect));

        let unoffered = [
            KeyCode::Char('x'),
            KeyCode::Char('a'),
            KeyCode::Right,
            KeyCode::Delete,
            KeyCode::Enter,
            KeyCode::Tab,
            KeyCode::Char('6'),
        ];

        for code in unoffered {
            let answer = handle(
                &Event::Key(KeyEvent::new(code, KeyModifiers::NONE)),
                &mut state,
                Focus::Region,
            );
            assert_eq!(answer, Reply::Ignored, "{code:?} was not ignored");
            assert_eq!(
                state.sub_mode(),
                Some(crate::diagram_state::SubMode::StrokeSelect)
            );
        }
    }

    #[test]
    fn unclaimed_global_shortcuts_in_stroke_select_are_ignored() {
        let mut state = a_state_over(A_BOX);
        state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));
        state.set_sub_mode(Some(crate::diagram_state::SubMode::StrokeSelect));

        let global_shortcuts = [
            (KeyCode::Char('z'), KeyModifiers::CONTROL),
            (KeyCode::Char('s'), KeyModifiers::CONTROL),
            (KeyCode::Char('q'), KeyModifiers::CONTROL),
            (KeyCode::Char('n'), KeyModifiers::CONTROL),
            (KeyCode::Char('o'), KeyModifiers::CONTROL),
        ];

        for (code, modifiers) in global_shortcuts {
            let answer = handle(
                &Event::Key(KeyEvent::new(code, modifiers)),
                &mut state,
                Focus::Region,
            );
            assert_eq!(answer, Reply::Ignored, "{code:?} was not ignored");
        }
    }

    #[test]
    fn pressing_z_on_a_selected_figure_enters_order_mode() {
        for fixture in [A_BOX, A_LINE, A_CONNECTOR] {
            let mut state = a_state_over(fixture);
            state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));

            let answer = handle(
                &Event::Key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE)),
                &mut state,
                Focus::Region,
            );
            assert_eq!(answer, Reply::Repaint);
            assert_eq!(
                state.sub_mode(),
                Some(crate::diagram_state::SubMode::OrderMode)
            );
        }
    }

    #[test]
    fn pressing_z_in_idle_is_ignored() {
        let mut state = a_state_over(A_BOX);
        assert_eq!(state.selected(), None);

        let answer = handle(
            &Event::Key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE)),
            &mut state,
            Focus::Region,
        );
        assert_eq!(answer, Reply::Ignored);
        assert_eq!(state.sub_mode(), None);
    }

    #[test]
    fn page_up_in_order_mode_dispatches_forward() {
        let mut state = a_state_over(A_BOX);
        let id = ShapeId::new(NonZeroU16::new(1).unwrap());
        state.select(Some(id));
        state.set_sub_mode(Some(crate::diagram_state::SubMode::OrderMode));

        let answer = handle(
            &Event::Key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE)),
            &mut state,
            Focus::Region,
        );
        assert_eq!(answer, Reply::Act(Action::Command(Command::Forward { id })));
        assert_eq!(
            state.sub_mode(),
            Some(crate::diagram_state::SubMode::OrderMode)
        );
    }

    #[test]
    fn page_down_in_order_mode_dispatches_backward() {
        let mut state = a_state_over(A_BOX);
        let id = ShapeId::new(NonZeroU16::new(1).unwrap());
        state.select(Some(id));
        state.set_sub_mode(Some(crate::diagram_state::SubMode::OrderMode));

        let answer = handle(
            &Event::Key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)),
            &mut state,
            Focus::Region,
        );
        assert_eq!(
            answer,
            Reply::Act(Action::Command(Command::Backward { id }))
        );
        assert_eq!(
            state.sub_mode(),
            Some(crate::diagram_state::SubMode::OrderMode)
        );
    }

    #[test]
    fn home_in_order_mode_dispatches_to_front() {
        let mut state = a_state_over(A_BOX);
        let id = ShapeId::new(NonZeroU16::new(1).unwrap());
        state.select(Some(id));
        state.set_sub_mode(Some(crate::diagram_state::SubMode::OrderMode));

        let answer = handle(
            &Event::Key(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)),
            &mut state,
            Focus::Region,
        );
        assert_eq!(answer, Reply::Act(Action::Command(Command::ToFront { id })));
        assert_eq!(
            state.sub_mode(),
            Some(crate::diagram_state::SubMode::OrderMode)
        );
    }

    #[test]
    fn end_in_order_mode_dispatches_to_back() {
        let mut state = a_state_over(A_BOX);
        let id = ShapeId::new(NonZeroU16::new(1).unwrap());
        state.select(Some(id));
        state.set_sub_mode(Some(crate::diagram_state::SubMode::OrderMode));

        let answer = handle(
            &Event::Key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE)),
            &mut state,
            Focus::Region,
        );
        assert_eq!(answer, Reply::Act(Action::Command(Command::ToBack { id })));
        assert_eq!(
            state.sub_mode(),
            Some(crate::diagram_state::SubMode::OrderMode)
        );
    }

    #[test]
    fn escape_in_order_mode_leaves_the_sub_mode_and_keeps_the_selection() {
        let mut state = a_state_over(A_BOX);
        let id = ShapeId::new(NonZeroU16::new(1).unwrap());
        state.select(Some(id));
        state.set_sub_mode(Some(crate::diagram_state::SubMode::OrderMode));

        let answer = handle(
            &Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            &mut state,
            Focus::Region,
        );
        assert_eq!(answer, Reply::Repaint);
        assert_eq!(state.sub_mode(), None);
        assert_eq!(state.selected(), Some(id));
    }

    #[test]
    fn unoffered_keys_in_order_mode_are_ignored() {
        let mut state = a_state_over(A_BOX);
        let id = ShapeId::new(NonZeroU16::new(1).unwrap());
        state.select(Some(id));
        state.set_sub_mode(Some(crate::diagram_state::SubMode::OrderMode));

        let unoffered = [
            KeyCode::Right,
            KeyCode::Left,
            KeyCode::Delete,
            KeyCode::Enter,
            KeyCode::Tab,
            KeyCode::Char('s'),
            KeyCode::Char('z'),
            KeyCode::Char('6'),
            KeyCode::F(1),
        ];

        for code in unoffered {
            let answer = handle(
                &Event::Key(KeyEvent::new(code, KeyModifiers::NONE)),
                &mut state,
                Focus::Region,
            );
            assert_eq!(answer, Reply::Ignored, "{code:?} was not ignored");
            assert_eq!(
                state.sub_mode(),
                Some(crate::diagram_state::SubMode::OrderMode)
            );
        }
    }
}
