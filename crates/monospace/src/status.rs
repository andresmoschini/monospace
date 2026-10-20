//! The status bar along the bottom.
//!
//! **It says what the application is until the editing surface gives it something to say.** The
//! name and the version are what a person looking at the terminal can be told at once, and the
//! state of the session — what has been changed and what can be undone — is #181's.

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

/// Paints the status bar.
pub(crate) fn draw(area: Rect, buf: &mut Buffer, state: &mut StatusLineState, theme: &SalsaTheme) {
    let style: StatusLineStyle = theme.style(WidgetStyle::STATUSLINE);
    let version = u16::try_from(VERSION.len()).expect("a version fits a line");

    StatusLine::new()
        .layout([Constraint::Fill(1), Constraint::Length(version)])
        .styles(style)
        .render(area, buf, state);
}
