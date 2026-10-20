//! What the application calls its events, its actions and its failures.
//!
//! **Three words, and they are not the same thing.** An *event* is what the application is told:
//! the framework's own events, and the terminal's carried as they arrived. An *action* is what
//! the application decides to do about one. A *failure* is what it reports when it cannot carry
//! on. The three are kept apart because the rule the whole application rests on is that an event
//! becomes an action and the action is the only thing that changes the state — and a word that
//! means two of those at once cannot say which rule is being held.

use ratatui_crossterm::crossterm::event::Event;

/// What the application reports when it cannot carry on.
pub(crate) type AppError = Box<dyn std::error::Error>;

/// Everything the application is told.
#[derive(Debug, PartialEq)]
pub(crate) enum AppEvent {
    /// An action the chrome or the application's own keys ask for.
    Action(Action),

    /// An event a widget may want, carried as it arrived.
    Input(Event),

    /// The terminal changed size.
    Resized,

    /// The loop finished drawing a frame.
    Rendered,

    /// Neither the application's nor a widget's.
    Ignored,
}

/// What the application does about an event.
///
/// **Two meanings, and one way out.** The way out is what a person asks for; the forced exit is
/// what the terminal asks for when it is interrupted. They are separate actions because they are
/// separate meanings, and they end the loop the same way only because nothing has been edited
/// yet: the way out grows a pop-up to confirm and the forced exit does not, which is #189's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    /// The way out: the application ends and the terminal comes back.
    Quit,

    /// The forced exit: the application ends whatever the state is.
    ForceQuit,
}
