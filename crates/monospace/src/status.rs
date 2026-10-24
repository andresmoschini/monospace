//! The status bar along the bottom.
//!
//! **It says what the application is until the editing surface gives it something to say.** The
//! name and the version are what a person looking at the terminal can be told at once, and the
//! state of the session — what has been changed and what can be undone — is #181's.
//!
//! **One table says what the bar holds and how much room each part of it takes.** The state and
//! the layout have to agree, and they used to be written in two files: the state was built
//! section by section in `state`, and the widths were laid out here, so a section added to one of
//! them alone was drawn wrong or not at all, in silence. Both now come from the same list.

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

/// The sections of the bar, in the order they are drawn: how much room each one takes, and what it
/// says.
///
/// **The name takes what is left and the version takes what it needs**, so a screen too narrow for
/// both gives the version its room and the name whatever remains.
fn sections() -> [(Constraint, &'static str); 2] {
    let version = u16::try_from(VERSION.len()).expect("a version fits a line");

    [
        (Constraint::Fill(1), NAME),
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

/// Paints the status bar.
pub(crate) fn draw(area: Rect, buf: &mut Buffer, state: &mut StatusLineState, theme: &SalsaTheme) {
    let style: StatusLineStyle = theme.style(WidgetStyle::STATUSLINE);

    StatusLine::new()
        .layout(sections().map(|(constraint, _)| constraint))
        .styles(style)
        .render(area, buf, state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    /// The theme the chrome is drawn in.
    fn theme() -> SalsaTheme {
        Config::default().theme
    }

    /// The bar, as text.
    fn row(width: u16) -> String {
        let area = Rect::new(0, 0, width, 1);
        let mut buffer = Buffer::empty(area);
        draw(area, &mut buffer, &mut state(), &theme());

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
            row.contains(VERSION),
            "the bar does not give the version: {row:?}"
        );
    }

    #[test]
    fn the_state_has_one_section_per_row_of_the_table() {
        assert_eq!(state().status.len(), sections().len());
    }
}
