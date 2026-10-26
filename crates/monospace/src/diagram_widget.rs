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
use ratatui_widgets::block::Block;

use monospace_core::{GlyphCatalog, Offset};
use ratatui_core::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Style,
    widgets::Widget,
};
use ratatui_crossterm::crossterm::event::{Event, MouseButton, MouseEvent, MouseEventKind};

use crate::state::DiagramState;
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
}

/// What an event means to the region.
///
/// **The region answers a click and no key yet** (rule 3): what it was not asked about is answered
/// with nothing so that the event goes on to the next handler.
pub(crate) fn handle(raw: &Event, state: &mut DiagramState) -> Option<Action> {
    match raw {
        Event::Mouse(mouse) => click(*mouse, state),
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
    use ratatui_crossterm::crossterm::event::{KeyModifiers, MouseEvent};

    /// The region these tests draw into. **Its interior is `18×6` at `(1, 1)` of the screen**, which
    /// is what says where a click's column and row land.
    const REGION: Rect = Rect::new(0, 0, 20, 8);

    /// A description whose only shape is a light 4×3 box at `(2, 1)` of the window.
    const A_BOX: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 },
                      "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
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
        let _ = handle(&click(at.x, at.y), &mut state);
        assert!(state.selected().is_some(), "the box was not selected");

        for button in [MouseButton::Right, MouseButton::Middle] {
            assert_eq!(
                handle(&press(button, at.x, at.y), &mut state),
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
        assert_eq!(handle(&released, &mut state), None, "a release was a press");

        assert!(
            state.selected().is_some(),
            "a press that is not the left button dropped the selection"
        );
    }
}
