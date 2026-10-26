//! What the application calls its actions and its failures.
//!
//! **Two words, and they are not the same thing.** An *action* is what the application decides to
//! do about an event; a *failure* is what the application reports when it cannot carry on. The two
//! are kept apart because the rule the whole application rests on is that an event becomes an
//! action and the action is the only thing that changes the state — and a failure ends the loop
//! instead of becoming one.
//!
//! **An action need not change anything.** `Action::ScreenChanged` is the screen being owed a
//! paint, and it is the one action that never reaches the state; `Action::Delete` takes a shape out
//! where one is selected and owes a paint either way; `Action::Undo` and `Action::Redo` walk the
//! session's history and owe a paint whether or not there is a step to give back; the two focus
//! actions move the keyboard between the leaves and owe one too; and `Action::Quit` and
//! `Action::ForceQuit` end the loop.
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
/// **Every action but one is a meaning.** The way out and the forced exit end the loop; `Delete`
/// takes a shape out, and `Undo` and `Redo` give a step back; `About` opens the chrome's dialog and
/// the two focus actions say where the keyboard is; `ScreenChanged` is the screen being owed a paint
/// and nothing else. **The two exits are separate actions because they are separate meanings**, and
/// they end the loop the same way only because nothing has been edited yet: the way out grows a
/// pop-up to confirm and the forced exit does not, which is #189's.
///
/// **A meaning that is carried out leaves the keyboard on the region** (rule 13 of #189), and the
/// drop-down the bar was showing closes with it. Only three things are not that: the screen
/// changing, which is not a meaning; the bar asking for the keyboard, which is the one meaning that
/// puts it somewhere else; and the two exits, which are not carried out at all. **So a meaning does
/// not say where the keyboard is left** — `callbacks::apply` says it once — which is why this is a
/// table of meanings and not a table of leaves.
///
/// **A change of the screen is not a meaning of a person's, and undo will never hold one.** It is
/// an action because an event becomes one thing, and what the application does about a screen that
/// has changed is paint it again; see _The action model_ in
/// [`docs/application-model.md`](../../../docs/application-model.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    /// It changed what the screen shows, and nothing else: the frame is owed again.
    ScreenChanged,

    /// The step behind the session is given back, and the frame is owed either way.
    ///
    /// **A press with nothing behind walks nothing**: the session answers with itself, and the
    /// action still paints, which is the silence at the oldest position (D2 of #181).
    Undo,

    /// The step ahead of the session is given back, and the frame is owed either way.
    ///
    /// **A press with nothing ahead walks nothing**, and the action still paints.
    Redo,

    /// The selected shape is taken out of the session, and the frame is owed either way.
    ///
    /// **The meaning and not the identity.** The shape is read from the state where the action is
    /// carried out, so the key is one row of the shortcut table rather than a second way of
    /// naming what a click selected. **A deletion with nothing selected removes nothing and still
    /// paints**: carrying an action out owes the frame, which is what closes the menu's entry on
    /// the screen when there was no figure to take.
    Delete,

    /// The About dialog is opened, and the frame is owed.
    ///
    /// **The chrome's own and no session** (D5 of #189): it opens an overlay and records no step,
    /// so the drawing and its history are untouched.
    About,

    /// The keyboard goes to the menu bar, and the frame is owed.
    ///
    /// **The bar's key and not the bar's move** (D4 of #189): `F10` asks for the keyboard from
    /// either leaf, and `Alt` plus a navigation character brings it with the press. The bar says so,
    /// and carrying the action out is where the focus moves.
    FocusBar,

    /// The keyboard goes back to the region, and the frame is owed.
    ///
    /// **The meaning that says only that** (rule 13 of #189): the `Esc` with no drop-down left to
    /// close answers it, and every other meaning that is carried out leaves the keyboard there
    /// without saying so.
    FocusRegion,

    /// The way out: the application ends and the terminal comes back.
    Quit,

    /// The forced exit: the application ends whatever the state is.
    ForceQuit,
}
