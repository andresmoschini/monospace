//! What the application calls its actions, its answers and its failures.
//!
//! **Three words, and they are not the same thing.** An *action* is what the application decides
//! to do about an event; an *answer* is what a handler says about the event it was asked about;
//! a *failure* is what the application reports when it cannot carry on. The three are kept apart
//! because the rule the whole application rests on is that an event becomes an action and the
//! action is the only thing that changes the state — and a word that means two of those at once
//! cannot say which rule is being held.
//!
//! The event itself is not here: it is what the loop polls, and it lives in `crate::event`
//! beside the two translations into it.

/// What the application reports when it cannot carry on.
pub(crate) type AppError = Box<dyn std::error::Error>;

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

/// What a handler answers about an event it was asked to look at.
///
/// **A handler does not act.** It says what the event was worth to it, and the caller turns the
/// answer into an action or into a request to repaint, so that an entry of the menu and the key
/// that means the same thing are decided in one place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Answer {
    /// It took no interest in the event.
    Nothing,

    /// It changed something the screen shows.
    Repaint,

    /// The event means an action.
    Action(Action),
}
