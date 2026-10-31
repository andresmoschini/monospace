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
/// **Every action but one is a meaning.** The way out and the forced exit end the loop; `Delete`
/// takes a shape out, and `Undo` and `Redo` give a step back; `About` opens the chrome's dialog;
/// `ScreenChanged` is the screen being owed a paint and nothing else. **The two exits are separate
/// actions because they are separate meanings**, and they end the loop the same way only because
/// nothing has been edited yet: the way out grows a pop-up to confirm and the forced exit does not,
/// which is #189's.
///
/// **A meaning that is carried out leaves the keyboard on the region** (rule 13 of #189), and the
/// drop-down the bar was showing closes with it. Only two things are not that: the screen changing,
/// which is not a meaning, and the two exits, which are not carried out at all. **So a meaning does
/// not say where the keyboard is left** — `callbacks::apply` says it once — which is why this is a
/// table of meanings and not a table of leaves.
///
/// **No action names a leaf** (D1 of #189): the keyboard moves where the event is addressed — a
/// press by its position, the bar's own three keys inside [`crate::menu::handle`] — so there is no
/// action for reaching the bar and none for leaving it.
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

    /// The selected shape is displaced by `(dx, dy)` cells, and the frame is owed when one moved.
    ///
    /// **The meaning and not the identity**, as [`Self::Delete`] is: the shape is read from the
    /// state where the action is carried out, so the four arrows are one action carrying a delta
    /// rather than four actions naming a figure each. **A move with nothing selected moves nothing
    /// and owes no frame**: nothing was carried out, so there is nothing to paint.
    Move(i16, i16),

    /// The About dialog is opened, and the frame is owed.
    ///
    /// **The chrome's own and no session** (D5 of #189): it opens an overlay and records no step,
    /// so the drawing and its history are untouched.
    About,

    /// The way out: the application ends and the terminal comes back.
    Quit,

    /// The forced exit: the application ends whatever the state is.
    ForceQuit,
}

/// What a handler says about an event.
///
/// **The four answers are the protocol, and their order is [`Reply::merge`]'s.** `Ignored` lets
/// the next handler be asked; the other three stop the chain. `Consumed` is the event taken with
/// no frame owed, `Repaint` is the event taken with one, and `Act` carries a meaning for
/// `callbacks::apply` to carry out.
///
/// **"Not mine" and "mine, and nothing changed" are different answers**, which is what the
/// protocol this replaced could not say: `None` meant both, so a handler that had consumed an
/// event left the next one to answer it as well.
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
}

impl Reply {
    /// Combines this reply with what addressing the event did.
    ///
    /// **The order is `Act > Repaint > Consumed > Ignored`**: an action is carried out whatever
    /// else happened, a frame is owed by either answer, and a handler that took the event stops
    /// the chain even where addressing it changed nothing. **Addressing a press only ever says
    /// `Ignored` or `Repaint` today**, and the order is written for all four answers so that a
    /// change to what addressing says cannot be lost here in silence.
    pub(crate) fn merge(self, addressed: Reply) -> Reply {
        match (self, addressed) {
            (Reply::Ignored, addressed) => addressed,
            (Reply::Act(action), _) | (_, Reply::Act(action)) => Reply::Act(action),
            (Reply::Consumed | Reply::Repaint, Reply::Repaint) => Reply::Repaint,
            (this, _) => this,
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

    /// The four answers, one of each.
    fn every_reply() -> [Reply; 4] {
        [
            Reply::Ignored,
            Reply::Consumed,
            Reply::Repaint,
            Reply::Act(Action::Quit),
        ]
    }

    /// **`Ignored` is the only answer that gives way**, from either side of the merge.
    #[test]
    fn an_ignored_reply_gives_way_to_the_other_side() {
        for reply in every_reply() {
            assert_eq!(
                reply.clone().merge(Reply::Ignored),
                reply,
                "a handler's {reply:?} lost to the press's silence"
            );
            assert_eq!(
                Reply::Ignored.merge(reply.clone()),
                reply,
                "the press's silence beat the handler's {reply:?}"
            );
        }
    }

    /// **A frame owed by either side is owed**: a silence and a repaint merge with the repaint,
    /// and two of the same answer stay that answer.
    #[test]
    fn a_frame_owed_by_either_side_is_owed() {
        assert_eq!(Reply::Consumed.merge(Reply::Repaint), Reply::Repaint);
        assert_eq!(Reply::Repaint.merge(Reply::Consumed), Reply::Repaint);
        assert_eq!(Reply::Repaint.merge(Reply::Repaint), Reply::Repaint);
        assert_eq!(Reply::Consumed.merge(Reply::Consumed), Reply::Consumed);
    }

    /// **An action outweighs every other answer**, which is the top of the order the protocol
    /// promises: `Act > Repaint > Consumed > Ignored`.
    #[test]
    fn an_action_outweighs_every_other_answer() {
        let action = Reply::Act(Action::Quit);

        for reply in every_reply() {
            assert_eq!(
                action.clone().merge(reply.clone()),
                action,
                "the handler's action lost to {reply:?}"
            );
            assert_eq!(
                reply.clone().merge(action.clone()),
                action,
                "the press's action lost to {reply:?}"
            );
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
