//! What the application calls its events, its failures, and the foreign event it carries.
//!
//! The only module in `app/` that names a terminal library, and it does so in one line.

use ratatui::crossterm::event::Event;

/// What the application reports when it cannot carry on.
pub(crate) type AppError = Box<dyn std::error::Error>;

/// Everything the application is told.
#[derive(Debug, PartialEq)]
pub(crate) enum AppEvent {
    Quit,

    Increment,

    About,

    /// An event a widget may want, carried as it arrived. Nothing consumes it yet.
    #[allow(
        dead_code,
        reason = "the widget layer it exists for does not exist yet"
    )]
    Input(Event),

    /// The terminal changed size.
    Resized,

    /// The loop finished drawing a frame.
    Rendered,

    /// Neither the application's nor a widget's.
    Ignored,
}
