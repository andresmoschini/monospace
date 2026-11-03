//! What the application calls its actions and its failures.
//!
//! **Two words, and they are not the same thing.** An *action* is what the application decides to
//! do about an event; a *failure* is what the application reports when it cannot carry on. The two
//! are kept apart because the rule the whole application rests on is that an event becomes a reply
//! and the action the reply carries is the only thing that changes the session — and a failure ends
//! the loop instead of becoming one.
//!
//! **An action need not change anything.** `Action::Delete` takes a shape out where one is
//! selected and owes a paint either way; `Action::Move` displaces what is selected and owes a
//! paint whether or not there was one; `Action::Undo` and `Action::Redo` walk the session's
//! history and owe a paint whether or not there is a step to give back; and `Action::About` and
//! `Action::ShowError` open the chrome's dialog. **The way out is not an action**: nothing about
//! the state changes, so there is nothing to carry out, and it is [`Reply::Quit`], which says
//! whether it is the forced one.
//!
//! **A handler says an action rather than doing one**, so that an entry of the menu and the key
//! that means the same thing are decided in one place. A handler answers with a [`Reply`], which
//! keeps "not mine" apart from "mine, and nothing changed": only [`Reply::Ignored`] lets the next
//! handler be asked.
//!
//! The event itself is not here: it is what the loop polls, and it lives in `crate::event`
//! beside the two translations into it.

/// What the application reports when it cannot carry on.
pub(crate) type AppError = Box<dyn std::error::Error>;

/// What the application does about an event.
///
/// **Every action is a meaning the state must show.** `Delete` takes a shape out, `Undo` and
/// `Redo` give a step back, `Move` displaces what is selected, and `About` and `ShowError` open the
/// chrome's dialog. **Ending the application is not one of them**: it changes no state, so there is
/// nothing to carry out, and it is an answer of the [`Reply`]'s own instead.
///
/// **A meaning that is carried out leaves the keyboard on the region** (rule 13 of #189), and the
/// drop-down the bar was showing closes with it. **So a meaning does not say where the keyboard is
/// left** — `callbacks::apply` says it once — which is why this is a table of meanings and not a
/// table of leaves.
///
/// **No action names a leaf** (D1 of #189): the keyboard moves where the event is addressed — a
/// press by its position, the bar's own three keys inside [`crate::menu::handle`] — so there is no
/// action for reaching the bar and none for leaving it.
///
/// **A change of the screen is not a meaning of a person's, and undo will never hold one.** It is
/// what [`Reply::Repaint`] says instead; see _The action model_ in
/// [`docs/application-model.md`](../../../docs/application-model.md).
///
/// **It is not `Copy`**: a meaning may carry what a person is told, and that text is owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Action {
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

    /// The selected shape is displaced by `(dx, dy)` cells, and the frame is owed either way.
    ///
    /// **The meaning and not the identity**, as [`Self::Delete`] is: the shape is read from the
    /// state where the action is carried out, so the four arrows are one action carrying a delta
    /// rather than four actions naming a figure each. **A move with nothing selected moves
    /// nothing**: the action is applied all the same, and `apply` continues to accept it, although
    /// the region no longer emits it without a selection.
    Move(i16, i16),

    /// The selected shape is resized by `(dx, dy)` cells, and the frame is owed either way.
    ///
    /// **The meaning and not the identity**, as [`Self::Move`] is: the shape is read from the
    /// state where the action is carried out.
    Resize(i16, i16),

    /// The About dialog is opened, and the frame is owed.
    ///
    /// **The chrome's own and no session** (D5 of #189): it opens an overlay and records no step,
    /// so the drawing and its history are untouched.
    About,

    /// A fresh session with an empty diagram is started.
    ///
    /// **Carried out immediately without confirmation** (D3 of #273), discarding current work,
    /// selection, and undo history.
    New,

    /// A fresh session with the built-in demonstration description is started.
    ///
    /// **Carried out immediately without confirmation** (D3 of #273), discarding current work,
    /// selection, and undo history.
    OpenDemo,

    /// The open dialog is opened, and the frame is owed.
    ///
    /// **The chrome's own and no session**, as [`Self::About`] is: opening the dialog records no
    /// step, and the path it is given is read only when [`Self::OpenFile`] is taken.
    Open,

    /// A description is read from disk and starts a fresh session.
    ///
    /// **The path is carried by the action**, and reading and parsing are done when the action is
    /// carried out. **A failure changes no session**: the error is reported via
    /// [`Self::ShowError`] and the current session is untouched. **A success starts a fresh
    /// session**, discarding the previous one without confirmation.
    OpenFile(String),

    /// The save dialog is opened, and the frame is owed.
    ///
    /// **The chrome's own and no session**, as [`Self::Open`] is: opening the dialog records no
    /// step, leaves the keyboard on the region, and saves only when [`Self::SaveFile`] is taken.
    Save,

    /// The current session's diagram is serialized and written to disk at this path.
    ///
    /// **The path is carried by the action**, and serialization and writing are done when the
    /// action is carried out. **A failure changes no session**: the error is reported via
    /// [`Self::ShowError`] and the current session is untouched. **A success leaves the current
    /// session untouched**, overwriting any existing file without confirmation.
    SaveFile(String),

    /// The error dialog is opened over this text, and the frame is owed.
    ///
    /// **The text is whoever failed's**, so it is owned and not a word of a table. Like
    /// [`Self::About`], it is the chrome's own and no session: it opens the overlay, replacing
    /// whatever it held, and records no step.
    ShowError(String),
}

/// What a handler says about an event.
///
/// **The five answers are the protocol.** `Ignored` lets the next handler be asked; the other four
/// stop the chain. `Consumed` is the event taken with no frame owed, `Repaint` is the event taken
/// with one, `Act` carries a meaning for `callbacks::apply` to carry out, and `Quit` ends the loop:
/// nothing about the state changes, so there is nothing to apply and no frame to owe.
///
/// **"Not mine" and "mine, and nothing changed" are different answers**, which is what the
/// protocol this replaced could not say: `None` meant both, so a handler that had consumed an
/// event left the next one to answer it as well. **A change of the screen is not a meaning of a
/// person's**, so it is `Repaint` rather than a variant of [`Action`].
///
/// **It travels by value but is not `Copy`**: an answer can carry an [`Action`] that owns text, so
/// [`and_repaint`](Self::and_repaint) and [`or_else`](Self::or_else) take it and give it back
/// rather than borrowing it, and a caller that keeps one clones it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Reply {
    /// Not mine: ask the next handler.
    Ignored,

    /// Mine, and nothing changed: the chain stops and no frame is owed.
    Consumed,

    /// Mine, and the frame is owed.
    Repaint,

    /// Mine, and this action is carried out before the frame is painted.
    Act(Action),

    /// The way out: the application ends and the terminal comes back.
    ///
    /// **It is not an [`Action`]**: nothing about the state changes, so there is nothing to carry
    /// out. **The two ways out are one answer and a bit**: `forced` is the difference between them,
    /// and it changes nothing about what happens — the way out grows a pop-up to confirm and the
    /// forced exit does not, which is #189's, and until then both end the loop the same way. **The
    /// bit is carried here rather than read back off the shortcut's reach** (D6 of #189), because
    /// the row that read the key is gone by the time the answer reaches the loop, and the answer is
    /// what #189 will ask.
    ///
    /// **A forced exit is answered before any handler**: it is the one request the chrome and the
    /// dialog may not swallow, so it is `shortcuts`' first reach, and the one answer nothing else
    /// may be added to.
    Quit { forced: bool },
}

impl Reply {
    /// The way out, as the menu's entry and the shortcut it shows answer it.
    pub(crate) const fn quit() -> Reply {
        Reply::Quit { forced: false }
    }

    /// The forced exit, as `Ctrl+C` answers it.
    pub(crate) const fn force_quit() -> Reply {
        Reply::Quit { forced: true }
    }

    /// This answer, raised to a frame where it owes none.
    ///
    /// **It raises and never lowers.** Only an answer that owes no frame becomes `Repaint`; an
    /// action and an exit say more than a paint, so they are left exactly as they are. **The one
    /// thing this is for is something else having changed the screen** — the keyboard moving to
    /// where a press landed, the label the bar reversed, the drop-down it opened — and that is
    /// something the answer knows nothing about.
    ///
    /// **`Ignored` is raised too**, because a handler that answered nothing can still be an event
    /// the screen moved under; whether it did is the caller's to know, and the bar asks only where
    /// the widget answered, so an event still going down the chain keeps its silence.
    ///
    /// **There is no order here, and that is the point.** Nothing weighs one answer against
    /// another, so two answers meeting cannot resolve by which side they came in on: what a change
    /// of the screen adds to an answer is a frame, whatever the answer was.
    pub(crate) fn and_repaint(self) -> Reply {
        match self {
            Reply::Ignored | Reply::Consumed => Reply::Repaint,
            this => this,
        }
    }

    /// Asks `next` only where this handler ignored the event.
    pub(crate) fn or_else(self, next: impl FnOnce() -> Reply) -> Reply {
        match self {
            Reply::Ignored => next(),
            this => this,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    /// Every answer, with the way out in both of its roads.
    fn every_reply() -> [Reply; 6] {
        [
            Reply::Ignored,
            Reply::Consumed,
            Reply::Repaint,
            Reply::Act(Action::About),
            Reply::quit(),
            Reply::force_quit(),
        ]
    }

    /// **Only silence is raised to a frame**, and every answer is walked: `Ignored` and `Consumed`
    /// owe one after the raise, and an action and an exit are left exactly as they were.
    #[test]
    fn only_silence_is_raised_to_a_frame() {
        for reply in every_reply() {
            let raised = reply.clone().and_repaint();

            match reply {
                Reply::Ignored | Reply::Consumed => {
                    assert_eq!(raised, Reply::Repaint, "{reply:?} was not raised");
                }
                _ => assert_eq!(raised, reply, "{reply:?} was lowered or changed"),
            }
        }
    }

    /// **Only `Ignored` asks the next handler**, so an event that was taken is never answered
    /// twice.
    #[test]
    fn only_an_ignored_reply_asks_the_next_handler() {
        for reply in every_reply() {
            let asked = Cell::new(false);
            let next = || {
                asked.set(true);
                Reply::Repaint
            };

            let answer = reply.clone().or_else(next);

            if reply == Reply::Ignored {
                assert!(asked.get(), "an ignored event stopped the chain");
                assert_eq!(answer, Reply::Repaint);
            } else {
                assert!(!asked.get(), "{reply:?} asked the next handler");
                assert_eq!(answer, reply);
            }
        }
    }
}
