//! What the application calls its actions and its failures.
//!
//! **Two words, and they are not the same thing.** An *action* is what the application decides to
//! do about an event; a *failure* is what the application reports when it cannot carry on. The two
//! are kept apart because the rule the whole application rests on is that an event becomes an
//! action and the action is the only thing that changes the state — and a failure ends the loop
//! instead of becoming one.
//!
//! **An action need not change anything.** `Action::ScreenChanged` is the screen being owed a
//! paint, and it is the one action that never reaches the state; the other two end the loop.
//!
//! **A handler says an action rather than doing one**, so that an entry of the menu and the key
//! that means the same thing are decided in one place. A handler that took no interest says
//! `None`, and the handlers are asked in order until one answers.
//!
//! The event itself is not here: it is what the loop polls, and it lives in `crate::event`
//! beside the two translations into it.

/// What the application reports when it cannot carry on.
pub(crate) type AppError = Box<dyn std::error::Error>;

/// What the application does about an event.
///
/// **Two meanings and one change of the screen.** The way out is what a person asks for; the
/// forced exit is what the terminal asks for when it is interrupted; `ScreenChanged` is what the
/// screen is owed when an event changed it and nothing else. The way out and the forced exit are
/// separate actions because they are separate meanings, and they end the loop the same way only
/// because nothing has been edited yet: the way out grows a pop-up to confirm and the forced exit
/// does not, which is #189's.
///
/// **A change of the screen is not a meaning of a person's, and undo will never hold one.** It is
/// an action because an event becomes one thing, and what the application does about a screen that
/// has changed is paint it again; see _The action model_ in
/// [`docs/application-model.md`](../../../docs/application-model.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    /// It changed what the screen shows, and nothing else: the frame is owed again.
    ScreenChanged,

    /// The way out: the application ends and the terminal comes back.
    Quit,

    /// The forced exit: the application ends whatever the state is.
    ForceQuit,
}
