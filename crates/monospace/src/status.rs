//! The status bar along the bottom.
//!
//! **It says what the application is until the editing surface gives it something to say.** The
//! name and the version are what a person looking at the terminal can be told at once, and the
//! state of the session — what has been changed and what can be undone — is #181's.
//!
//! **It also says how far the drawing reaches**, which is the diagram's footprint in the plane
//! rather than its picture in the region: a figure behind another and one scrolled out of sight
//! both count, which is why the measurement is a [`FootprintSink`](monospace_core::FootprintSink)
//! between the drawing and the buffer. The section is written from the [`Footprint`] the frame is
//! drawn with, so it is the last frame's answer and never the one before it.
//!
//! **One table says what the bar holds and how much room each part of it takes.** The state and
//! the layout have to agree, and they used to be written in two files: the state was built
//! section by section in `state`, and the widths were laid out here, so a section added to one of
//! them alone was drawn wrong or not at all, in silence. Both now come from the same list.

use monospace_core::Footprint;
use rat_theme4::WidgetStyle;
use rat_theme4::theme::SalsaTheme;
use rat_widget::statusline::{StatusLine, StatusLineState, StatusLineStyle};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Rect};
use ratatui_core::widgets::StatefulWidget;

/// The name the application answers to.
pub(crate) const NAME: &str = "monospace";

/// The version of the application.
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

/// What the bar says about a drawing that has reached nothing yet.
const NOTHING_DRAWN: &str = "nothing drawn";

/// The index of the section that says how far the drawing reaches.
const FOOTPRINT: usize = 1;

/// The room that section takes.
///
/// **A fixed width, because the bar lays its sections out before it knows the text.** The longest
/// text there is — a drawing spanning the whole plane, `65536×65536 at -32768,-32768` — is longer
/// than this and is clipped to what fits; the alternative is a version that moves whenever the
/// drawing does, which is worse.
const FOOTPRINT_WIDTH: u16 = 18;

/// The sections of the bar, in the order they are drawn: how much room each one takes, and what it
/// says.
///
/// **The name takes what is left and the version takes what it needs**, so a screen too narrow for
/// all three gives the version its room and the name whatever remains. **The footprint's text is not
/// here**: it is a fact of the last frame rather than of the application, so [`draw`] writes it
/// from the drawing it is handed.
fn sections() -> [(Constraint, &'static str); 3] {
    let version = u16::try_from(VERSION.len()).expect("a version fits a line");

    [
        (Constraint::Fill(1), NAME),
        (Constraint::Length(FOOTPRINT_WIDTH), NOTHING_DRAWN),
        (Constraint::Length(version), VERSION),
    ]
}

/// The state of the bar: one section per row of the table, saying what the row says.
pub(crate) fn state() -> StatusLineState {
    let mut state = StatusLineState::default();

    for (at, (_, text)) in sections().into_iter().enumerate() {
        state.status(at, text);
    }

    state
}

/// How far the drawing reaches, as the bar says it: `46×13 at 0,0` or `4×3 at -4,-3`.
///
/// **The width and height are derived here rather than on [`Footprint`]**, because the span does not
/// fit a [`Size`] when a drawing reaches across the whole plane and only a caller that knows what
/// to do about it can decide. The arithmetic is `i32`: a `Pos` is `i16` and `max - min + 1` is
/// `i32` at most, so no corner of the plane overflows the sum.
///
/// [`Size`]: monospace_core::Size
fn footprint_text(footprint: Option<Footprint>) -> String {
    let Some(footprint) = footprint else {
        return NOTHING_DRAWN.to_owned();
    };

    let width = i32::from(footprint.max.x) - i32::from(footprint.min.x) + 1;
    let height = i32::from(footprint.max.y) - i32::from(footprint.min.y) + 1;

    format!(
        "{width}×{height} at {x},{y}",
        x = footprint.min.x,
        y = footprint.min.y
    )
}

/// Paints the status bar, saying how far the drawing of this frame reaches.
///
/// **The footprint is a parameter rather than a call**, so that a bar cannot be drawn without saying
/// what the drawing reached: the compiler asks every caller, and the section is never left saying
/// what the frame before it said.
pub(crate) fn draw(
    area: Rect,
    buf: &mut Buffer,
    state: &mut StatusLineState,
    footprint: Option<Footprint>,
    theme: &SalsaTheme,
) {
    let style: StatusLineStyle = theme.style(WidgetStyle::STATUSLINE);

    state.status(FOOTPRINT, footprint_text(footprint));

    StatusLine::new()
        .layout(sections().map(|(constraint, _)| constraint))
        .styles(style)
        .render(area, buf, state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use monospace_core::Pos;

    /// The theme the chrome is drawn in.
    fn theme() -> SalsaTheme {
        Config::default().theme
    }

    /// The bar, as text.
    fn row(width: u16) -> String {
        row_with(width, None)
    }

    /// The bar, as text, drawn with the footprint the frame reached.
    fn row_with(width: u16, footprint: Option<Footprint>) -> String {
        let area = Rect::new(0, 0, width, 1);
        let mut buffer = Buffer::empty(area);
        draw(area, &mut buffer, &mut state(), footprint, &theme());

        (area.left()..area.right())
            .map(|column| buffer[(column, area.top())].symbol().to_string())
            .collect()
    }

    /// **A section the table has is a section on the screen.** The two halves of this rule used to
    /// live in two files, and a section added to the state alone was drawn nowhere and said so to
    /// nobody.
    #[test]
    fn every_section_of_the_table_is_drawn() {
        let row = row(40);

        assert!(
            row.contains(NAME),
            "the bar does not name the application: {row:?}"
        );
        assert!(
            row.contains(NOTHING_DRAWN),
            "the bar says nothing about a drawing that reached nothing: {row:?}"
        );
        assert!(
            row.contains(VERSION),
            "the bar does not give the version: {row:?}"
        );
    }

    /// The bar says the drawing's width, height and corner, and the numbers are the footprint's:
    /// `max - min + 1` on each axis, so a one-cell drawing is `1×1` and not `0×0`.
    #[test]
    fn the_bar_says_how_far_the_drawing_reaches() {
        let row = row_with(
            40,
            Some(Footprint {
                min: Pos { x: 0, y: 0 },
                max: Pos { x: 45, y: 12 },
            }),
        );

        assert!(
            row.contains("46×13 at 0,0"),
            "the bar does not say the drawing's size and corner: {row:?}"
        );
    }

    /// A drawing at a negative corner says so and measures four by three rather than counting from
    /// the origin: `min` is where the drawing starts, not how far it is from `(0, 0)`.
    #[test]
    fn the_bar_says_a_corner_before_the_origin_as_it_is() {
        let row = row_with(
            40,
            Some(Footprint {
                min: Pos { x: -4, y: -3 },
                max: Pos { x: -1, y: -1 },
            }),
        );

        assert!(
            row.contains("4×3 at -4,-3"),
            "the bar does not say the drawing's size and corner: {row:?}"
        );
    }

    /// One stamp is `1×1` at its own position, which is the case a `max - min` without the `+ 1`
    /// would report as nothing at all.
    #[test]
    fn a_single_stamped_cell_is_one_by_one() {
        let at = Pos { x: 7, y: 2 };

        assert_eq!(
            footprint_text(Some(Footprint { min: at, max: at })),
            "1×1 at 7,2"
        );
    }

    /// The bar may be drawn before anything has been stamped, and says so rather than showing a
    /// rectangle at the origin that would be indistinguishable from a drawing that put one cell
    /// there.
    #[test]
    fn nothing_drawn_is_said_in_words() {
        assert_eq!(footprint_text(None), NOTHING_DRAWN);
    }

    /// **The section's width is fixed, so a text too long for it is clipped rather than paid for by
    /// the rest of the bar.** The plane's own corner is the longest text there is, and the version
    /// keeps its room while it is drawn: a width that followed the text would move the name's
    /// boundary whenever the drawing did.
    #[test]
    fn a_text_longer_than_the_section_is_clipped() {
        let row = row_with(
            40,
            Some(Footprint {
                min: Pos {
                    x: i16::MIN,
                    y: i16::MIN,
                },
                max: Pos {
                    x: i16::MAX,
                    y: i16::MAX,
                },
            }),
        );

        assert!(
            row.contains(VERSION),
            "a long footprint text pushed the version off the bar: {row:?}"
        );
        assert!(
            !row.contains("65536×65536 at -32768,-32768"),
            "the bar drew a text longer than the section that holds it: {row:?}"
        );
    }

    #[test]
    fn the_state_has_one_section_per_row_of_the_table() {
        assert_eq!(state().status.len(), sections().len());
    }
}
